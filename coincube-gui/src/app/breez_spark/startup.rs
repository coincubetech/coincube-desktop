//! The Spark load of an unlock, running beside the Vault's daemon start.
//!
//! After an unlock, Spark's `Init` (spawning the bridge, Breez
//! `SdkBuilder::build()`, its SQLite and the Spark network) and the Vault's
//! daemon start used to run one after the other. A [`SparkStartup`] lets the
//! unlock start the Spark load as its own task and hand it to the
//! [`crate::loader::Loader`], which starts the daemon at once and waits for
//! Spark only where it builds the App.
//!
//! Ownership decides the bridge's life:
//!
//! * Every clone shares one load; there is never a second bridge.
//! * Dropping the last clone aborts a load still under way. The aborted
//!   future drops its half-built client, whose child process is
//!   `kill_on_drop`, and its own copy of the unlock secret, which is
//!   `Zeroizing`.
//! * A finished load keeps its backend only while a clone holds it. Waiting
//!   with [`SparkStartup::settled`] never keeps the load or its backend
//!   alive.

use std::future::Future;
use std::sync::Arc;

use tokio::sync::watch;

use crate::app::wallets::SparkBackend;

/// What a Spark load produced: its backend, or `None` when the Cube has no
/// Spark wallet or the bridge failed (the App then opens without Spark).
pub type SparkOutcome = Option<Arc<SparkBackend>>;

/// One Spark load, shared by everyone waiting on it. See the module docs.
#[derive(Clone)]
pub struct SparkStartup {
    inner: Arc<Inner>,
}

struct Inner {
    /// `None` until the load has settled.
    outcome: watch::Receiver<Option<SparkOutcome>>,
    /// The load still running, if it was spawned.
    task: Option<tokio::task::AbortHandle>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

impl std::fmt::Debug for SparkStartup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self.outcome() {
            None => "loading",
            Some(Some(_)) => "loaded",
            Some(None) => "unavailable",
        };
        f.debug_tuple("SparkStartup").field(&state).finish()
    }
}

impl From<SparkOutcome> for SparkStartup {
    fn from(outcome: SparkOutcome) -> Self {
        Self::ready(outcome)
    }
}

impl SparkStartup {
    /// A load that has already settled.
    pub fn ready(outcome: SparkOutcome) -> Self {
        let (_settled, receiver) = watch::channel(Some(outcome));
        Self {
            inner: Arc::new(Inner {
                outcome: receiver,
                task: None,
            }),
        }
    }

    /// No Spark wallet.
    pub fn none() -> Self {
        Self::ready(None)
    }

    /// Run `load` as its own task on the current Tokio runtime, so it does
    /// not wait on, or hold up, whatever the caller does next.
    pub fn spawn(load: impl Future<Output = SparkOutcome> + Send + 'static) -> Self {
        let (settled, receiver) = watch::channel(None);
        let task = tokio::spawn(async move {
            let outcome = load.await;
            // Nobody may be left to read it: then the backend drops here.
            let _ = settled.send(Some(outcome));
        });
        Self {
            inner: Arc::new(Inner {
                outcome: receiver,
                task: Some(task.abort_handle()),
            }),
        }
    }

    /// The load's outcome once it has settled; `None` while it runs.
    pub fn outcome(&self) -> Option<SparkOutcome> {
        self.inner.outcome.borrow().clone()
    }

    /// Resolves once the load has settled, or was abandoned because every
    /// clone was dropped. It holds only a receiver, never the load: waiting
    /// on it does not keep a bridge alive.
    pub fn settled(&self) -> impl Future<Output = ()> + Send + 'static {
        let mut outcome = self.inner.outcome.clone();
        async move {
            let _ = outcome.wait_for(Option::is_some).await;
        }
    }

    /// Wait for the load here, for a caller with nothing to overlap it with.
    pub async fn finish(self) -> Self {
        self.settled().await;
        // `self` still holds the load, so it cannot have been abandoned.
        Self::ready(self.outcome().flatten())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// Set when the load's future is dropped, finished or not: what drops
    /// a half-built Spark client and its secret.
    struct DropFlag(Arc<AtomicBool>);

    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn a_ready_startup_is_settled_at_once() {
        let startup = SparkStartup::none();
        assert!(matches!(startup.outcome(), Some(None)));
        startup.settled().await;
        assert!(matches!(startup.finish().await.outcome(), Some(None)));
    }

    #[tokio::test]
    async fn clones_share_one_load_and_its_outcome() {
        let loads = Arc::new(AtomicUsize::new(0));
        let (release, gate) = tokio::sync::oneshot::channel::<()>();
        let startup = {
            let loads = loads.clone();
            SparkStartup::spawn(async move {
                loads.fetch_add(1, Ordering::SeqCst);
                let _ = gate.await;
                None
            })
        };
        let retry = startup.clone();
        tokio::task::yield_now().await;
        assert!(startup.outcome().is_none(), "still loading");
        release.send(()).unwrap();
        retry.settled().await;
        assert!(matches!(startup.outcome(), Some(None)));
        assert!(matches!(retry.outcome(), Some(None)));
        assert_eq!(loads.load(Ordering::SeqCst), 1, "never a second bridge");
    }

    #[tokio::test]
    async fn dropping_every_clone_aborts_the_load_even_while_someone_waits() {
        let dropped = Arc::new(AtomicBool::new(false));
        let startup = {
            let flag = DropFlag(dropped.clone());
            SparkStartup::spawn(async move {
                let _flag = flag;
                std::future::pending::<()>().await;
                None
            })
        };
        let retry = startup.clone();
        let waiting = startup.settled();
        drop((startup, retry));
        // The waiter did not keep the load alive: it resolves because the
        // load was dropped, not because it finished.
        tokio::time::timeout(std::time::Duration::from_secs(5), waiting)
            .await
            .expect("an abandoned load settles its waiters");
        assert!(dropped.load(Ordering::SeqCst), "the load was aborted");
    }

    #[tokio::test]
    async fn finish_waits_for_the_load() {
        let (release, gate) = tokio::sync::oneshot::channel::<()>();
        let startup = SparkStartup::spawn(async move {
            let _ = gate.await;
            None
        });
        let finishing = tokio::spawn(startup.finish());
        tokio::task::yield_now().await;
        assert!(!finishing.is_finished());
        release.send(()).unwrap();
        let finished = finishing.await.unwrap();
        assert!(matches!(finished.outcome(), Some(None)));
    }
}
