//! Publish each accepted Worldview edit to COINCUBE Mobile through the
//! signed-in Connect account. There is no opt-in: every signed-in account
//! receives the saved selection.

use super::{Message, OverlayOrigin, GUI};
use crate::app::{
    settings::preferences::{DevicePreferences, PreferenceChange},
    state::preferences::{Operation, SaveRequest},
};
use crate::services::coincube::worldview::{Error, Selection, Snapshot};
use iced::Task;
use std::collections::VecDeque;

type Context = (u32, u64, u64);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Binding {
    origin: OverlayOrigin,
    context: Context,
}

#[derive(Debug, Clone)]
struct Intent {
    binding: Binding,
    epoch: uuid::Uuid,
}

#[derive(Debug, Clone, Copy)]
enum Request {
    Read,
    Publish(Selection),
}

#[derive(Debug, Default)]
pub struct Sharing {
    binding: Option<Binding>,
    // Rebinding must never allow a second request while an old PUT is active.
    in_flight: Option<(Binding, uuid::Uuid)>,
    pending: Option<uuid::Uuid>,
    operation: Option<Request>,
    snapshot: Option<Snapshot>,
    /// The last request failed. Nothing retries it; the next accepted edit
    /// clears it and starts again from a fresh read.
    error: Option<Error>,
    desired: Option<Selection>,
    read_ready: bool,
    epoch: uuid::Uuid,
    save: Option<(SaveRequest, Option<Intent>)>,
    queued: VecDeque<(PreferenceChange, Option<Intent>)>,
}

impl Sharing {
    fn invalidate_intents(&mut self) {
        self.epoch = uuid::Uuid::new_v4();
        self.desired = None;
        self.read_ready = false;
    }

    fn bind(&mut self, binding: &Binding) {
        if self.binding.as_ref() != Some(binding) {
            let same_account = self
                .binding
                .as_ref()
                .is_some_and(|current| current.context.0 == binding.context.0);
            self.desired = None;
            self.read_ready = false;
            self.binding = Some(binding.clone());
            self.pending = None;
            self.operation = None;
            self.snapshot = None;
            if !same_account {
                self.error = None;
            }
        }
    }

    fn complete(&mut self, request: uuid::Uuid, result: Result<Snapshot, Error>) -> bool {
        if self.pending != Some(request) {
            return false;
        }
        self.pending = None;
        let operation = self.operation.take();
        match result {
            Ok(snapshot) => {
                self.read_ready = matches!(operation, Some(Request::Read));
                self.snapshot = Some(snapshot);
                self.error = None;
            }
            Err(error) => {
                // Lost replies and conflicts never trigger an automatic retry.
                self.invalidate_intents();
                self.snapshot = None;
                self.error = Some(error);
            }
        }
        true
    }
}

fn relevant(change: PreferenceChange) -> bool {
    matches!(change, PreferenceChange::Worldview(_))
}

impl GUI {
    fn worldview_binding(&self) -> Option<Binding> {
        let mut origin = self.overlay_origin(None)?;
        origin.focus = None;
        self.worldview_binding_for(&origin)
    }

    fn worldview_binding_for(&self, origin: &OverlayOrigin) -> Option<Binding> {
        let tab = self
            .panes
            .get(origin.pane)?
            .tabs
            .iter()
            .find(|tab| tab.id == origin.tab_id)?;
        if tab.preferences_source() != Some(origin.source) {
            return None;
        }
        let account = self.connect_panel(origin)?;
        if !account.is_authenticated() {
            return None;
        }
        Some(Binding {
            origin: origin.clone(),
            context: (
                account.user.as_ref()?.id,
                account.session_generation(),
                account.auth_epoch(),
            ),
        })
    }

    fn worldview_intent(&self) -> Option<Intent> {
        let binding = self.worldview_binding()?;
        Some(Intent {
            binding,
            epoch: self.worldview_sharing.epoch,
        })
    }

    pub(super) fn worldview_record_save(&mut self, request: &SaveRequest) {
        if let Operation::Save(change) = request.operation {
            if relevant(change) {
                let intent = self.worldview_intent();
                self.worldview_sharing.save = Some((request.clone(), intent));
            }
        }
    }

    pub(super) fn worldview_queue_save(&mut self, change: PreferenceChange) {
        if relevant(change) {
            let intent = self.worldview_intent();
            self.worldview_sharing.queued.push_back((change, intent));
        }
    }

    pub(super) fn worldview_start_queued_save(&mut self, request: &SaveRequest) {
        if let Operation::Save(change) = request.operation {
            if relevant(change) {
                let queued = self.worldview_sharing.queued.pop_front();
                let intent = queued
                    .filter(|(expected, _)| *expected == change)
                    .and_then(|(_, intent)| intent);
                self.worldview_sharing.save = Some((request.clone(), intent));
            }
        }
    }

    pub(super) fn worldview_saved(
        &mut self,
        request: &SaveRequest,
        applied: bool,
        before: Option<DevicePreferences>,
    ) {
        if !self
            .worldview_sharing
            .save
            .as_ref()
            .is_some_and(|(pending, _)| pending == request)
        {
            return;
        }
        let (_, intent) = self.worldview_sharing.save.take().unwrap();
        let Some(intent) = intent else {
            return;
        };
        if !applied
            || intent.epoch != self.worldview_sharing.epoch
            || self.worldview_binding_for(&intent.binding.origin).as_ref() != Some(&intent.binding)
        {
            return;
        }
        let Some(after) = self
            .preferences
            .accepted()
            .map(|accepted| accepted.preferences)
        else {
            return;
        };
        let selection = Selection::from(after);
        if before.map(Selection::from) != Some(selection) {
            // An attempted edit cannot displace a previously accepted intent.
            // Rebind only when the new active pair has actually been saved.
            self.worldview_sharing.bind(&intent.binding);
            // Keep reverts too: the cached snapshot can predate an in-flight PUT.
            self.worldview_sharing.desired = Some(selection);
            // A new accepted edit is a fresh attempt after an earlier failure.
            // It still re-reads before writing, so an uncertain write is safe.
            self.worldview_sharing.error = None;
        }
    }

    fn worldview_start(&mut self, binding: Binding, operation: Request) -> Task<Message> {
        if self.worldview_sharing.in_flight.is_some() {
            return Task::none();
        }
        let Some(client) = self
            .connect_panel(&binding.origin)
            .and_then(|panel| panel.authenticated_client())
        else {
            return Task::none();
        };
        let previous = self.worldview_sharing.snapshot.clone();
        let request = uuid::Uuid::new_v4();
        self.worldview_sharing.pending = Some(request);
        self.worldview_sharing.in_flight = Some((binding.clone(), request));
        self.worldview_sharing.operation = Some(operation);
        self.worldview_sharing.error = None;
        Task::perform(
            async move {
                match operation {
                    Request::Read => client.get_worldview().await,
                    Request::Publish(selection) => {
                        client
                            .share_worldview(&previous.ok_or(Error::Unavailable)?, Some(selection))
                            .await
                    }
                }
            },
            move |result| {
                Message::Worldview(binding.origin.clone(), binding.context, request, result)
            },
        )
    }

    pub(super) fn worldview_automatic(&mut self) -> Task<Message> {
        if self.preferences.is_busy() || self.preferences.has_queued_changes() {
            // A GET held across local work is no longer fresh enough to write.
            self.worldview_sharing.read_ready = false;
            return Task::none();
        }
        let sharing = &self.worldview_sharing;
        if sharing.in_flight.is_some() {
            return Task::none();
        }
        if sharing.error.is_some() || sharing.desired.is_none() {
            return Task::none();
        }
        let Some(binding) = sharing.binding.clone() else {
            return Task::none();
        };
        if self.worldview_binding_for(&binding.origin).as_ref() != Some(&binding) {
            self.worldview_sharing.invalidate_intents();
            return Task::none();
        }
        if self
            .preferences
            .accepted()
            .map(|saved| Selection::from(saved.preferences))
            != self.worldview_sharing.desired
        {
            self.worldview_sharing.invalidate_intents();
            return Task::none();
        }
        // Re-read before every write: the PUT is conditional on the current
        // ETag, which Mobile or another Desktop may have moved.
        if !sharing.read_ready {
            return self.worldview_start(binding, Request::Read);
        }
        self.worldview_sharing.read_ready = false;
        let Some(snapshot) = self.worldview_sharing.snapshot.as_ref() else {
            self.worldview_sharing.invalidate_intents();
            return Task::none();
        };
        let selection = self.worldview_sharing.desired.take().unwrap();
        if snapshot.selection == Some(selection) {
            return Task::none();
        }
        self.worldview_start(binding, Request::Publish(selection))
    }

    pub(super) fn worldview_complete(
        &mut self,
        origin: OverlayOrigin,
        context: Context,
        request: uuid::Uuid,
        result: Result<Snapshot, Error>,
    ) -> Task<Message> {
        let binding = Binding { origin, context };
        if !self
            .worldview_sharing
            .in_flight
            .as_ref()
            .is_some_and(|(owner, id)| owner == &binding && *id == request)
        {
            return Task::none();
        }
        self.worldview_sharing.in_flight = None;
        if self.worldview_sharing.binding.as_ref() != Some(&binding)
            || self.worldview_sharing.pending != Some(request)
        {
            // A new origin's edit waited for this request. It must read again
            // after success, or pause if the same account's write is uncertain.
            if let Err(error) = result {
                if self
                    .worldview_sharing
                    .binding
                    .as_ref()
                    .is_some_and(|current| current.context.0 == binding.context.0)
                {
                    self.worldview_sharing.invalidate_intents();
                    self.worldview_sharing.snapshot = None;
                    self.worldview_sharing.error = Some(error);
                }
            }
            return self.worldview_automatic();
        }
        if self.worldview_binding_for(&binding.origin).as_ref() != Some(&binding) {
            // A removed/locked origin must not strand its request on return.
            // Never clear a newer request or adopt the departed account's reply.
            if self.worldview_sharing.pending == Some(request) {
                self.worldview_sharing.invalidate_intents();
                self.worldview_sharing.pending = None;
                self.worldview_sharing.operation = None;
                self.worldview_sharing.snapshot = None;
                self.worldview_sharing.error = None;
            }
            return self.worldview_automatic();
        }
        if self.worldview_sharing.complete(request, result) {
            self.worldview_automatic()
        } else {
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_completions_cannot_replace_a_new_request() {
        let mut sharing = Sharing::default();
        let old = uuid::Uuid::new_v4();
        let new = uuid::Uuid::new_v4();
        sharing.pending = Some(new);
        sharing.complete(old, Err(Error::Locked));
        assert_eq!(sharing.pending, Some(new));
        assert_eq!(sharing.error, None);
        sharing.complete(new, Err(Error::Conflict));
        assert_eq!(sharing.pending, None);
        assert_eq!(sharing.error, Some(Error::Conflict));
        assert_eq!(sharing.snapshot, None);
    }
    fn with_gui(test: impl FnOnce(&mut GUI) + Send + 'static) {
        let _guard = crate::app::session::test_guard();
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                let _enter = runtime.enter();
                let path =
                    std::env::temp_dir().join(format!("worldview-auto-{}", uuid::Uuid::new_v4()));
                std::fs::create_dir_all(&path).unwrap();
                let root = crate::dir::CoincubeDirectory::new(path.clone());
                let mut gui = super::super::app_preferences_sheet_tests::launcher_gui(&root);
                set_account(&mut gui, 7);
                test(&mut gui);
                drop(gui);
                std::fs::remove_dir_all(path).unwrap();
            })
            .unwrap()
            .join()
            .unwrap();
    }

    fn set_account(gui: &mut GUI, id: u32) {
        let pane = gui.focus.unwrap();
        let super::super::tab::State::Home(home) =
            &mut gui.panes.get_mut(pane).unwrap().tabs[0].state
        else {
            panic!("home");
        };
        home.connect_account.user = Some(crate::services::coincube::User {
            id,
            email: "fixture@example.invalid".into(),
            email_verified: Some(true),
            display_name: None,
        });
        home.connect_account.step = crate::app::state::connect::ConnectFlowStep::Dashboard;
    }

    fn selected(gui: &GUI) -> Selection {
        Selection::from(gui.preferences.accepted().unwrap().preferences)
    }

    fn snapshot(selection: Option<Selection>) -> Snapshot {
        Snapshot {
            account_uuid: "82473444-c619-4c95-bc6e-ef2d9f692e8b".into(),
            revision: 1,
            selection,
            etag: "\"test-etag\"".into(),
        }
    }

    fn shared(gui: &mut GUI) {
        let binding = gui.worldview_binding().unwrap();
        gui.worldview_sharing.bind(&binding);
        gui.worldview_sharing.snapshot = Some(snapshot(Some(selected(gui))));
    }

    fn finish_save(gui: &mut GUI, request: SaveRequest) {
        let Operation::Save(change) = request.operation else {
            panic!("save");
        };
        let receipt = super::super::GlobalSettings::save_preference(&request.root, change).unwrap();
        drop(gui.update(Message::Preferences(request, Ok(receipt))));
    }

    fn edit(gui: &mut GUI, change: PreferenceChange) {
        drop(gui.change_preference(change));
        let request = gui.worldview_sharing.save.as_ref().unwrap().0.clone();
        finish_save(gui, request);
    }

    /// Saves a change sharing does not track, through the same completion path.
    fn edit_unrelated(gui: &mut GUI, change: PreferenceChange) {
        assert!(!relevant(change));
        let request = gui.preferences.begin_save(change).unwrap();
        assert!(gui.worldview_sharing.save.is_none());
        finish_save(gui, request);
    }

    fn remote(gui: &mut GUI, result: Result<Snapshot, Error>) {
        let (binding, request) = gui.worldview_sharing.in_flight.clone().unwrap();
        drop(gui.worldview_complete(binding.origin, binding.context, request, result));
    }

    fn another_home(gui: &mut GUI, account: Option<u32>) {
        let pane = gui.focus.unwrap();
        let mut home =
            super::super::home::Home::new_for_chain(gui.config.coincube_directory.clone(), None).0;
        if let Some(id) = account {
            home.connect_account.user = Some(crate::services::coincube::User {
                id,
                email: "fixture@example.invalid".into(),
                email_verified: Some(true),
                display_name: None,
            });
            home.connect_account.step = crate::app::state::connect::ConnectFlowStep::Dashboard;
        }
        gui.panes
            .get_mut(pane)
            .unwrap()
            .tabs
            .push(super::super::tab::Tab::new(
                2,
                super::super::tab::State::Home(home),
            ));
        gui.panes.get_mut(pane).unwrap().focused_tab = 1;
    }

    fn open_preferences(gui: &mut GUI) {
        let pane = gui.focus.unwrap();
        let state = gui.panes.get(pane).unwrap();
        let tab = state.tabs[state.focused_tab].id;
        drop(gui.update(Message::Pane(
            pane,
            super::super::pane::Message::tab(
                tab,
                super::super::tab::Message::OpenAppPreferences(
                    super::super::tab::PreferencesSource::Home,
                ),
            ),
        )));
    }

    #[test]
    fn reverting_during_a_write_keeps_the_latest_accepted_pair() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            let original = selected(gui);
            edit(gui, PreferenceChange::Worldview(Luke));
            let interim = selected(gui);
            remote(gui, Ok(snapshot(Some(original))));
            assert!(
                matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == interim)
            );
            edit(gui, PreferenceChange::Worldview(Samurai));
            assert_eq!(gui.worldview_sharing.desired, Some(original));
            remote(gui, Ok(snapshot(Some(interim))));
            assert!(matches!(
                gui.worldview_sharing.operation,
                Some(Request::Read)
            ));
            remote(gui, Ok(snapshot(Some(interim))));
            assert!(
                matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == original)
            );
            remote(gui, Ok(snapshot(Some(original))));
            assert!(gui.worldview_sharing.pending.is_none());
        });
    }

    #[test]
    fn account_changed_during_disk_save_cannot_receive_an_old_accounts_edit() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            drop(gui.change_preference(PreferenceChange::Worldview(Luke)));
            let save = gui.worldview_sharing.save.as_ref().unwrap().0.clone();
            set_account(gui, 8);
            finish_save(gui, save);
            assert!(gui.worldview_sharing.desired.is_none());
            assert!(gui.worldview_sharing.pending.is_none());
            edit(gui, PreferenceChange::Worldview(Suitcoiner));
            assert_eq!(gui.worldview_sharing.binding.as_ref().unwrap().context.0, 8);
            assert!(matches!(
                gui.worldview_sharing.operation,
                Some(Request::Read)
            ));
        });
    }

    #[test]
    fn background_refresh_keeps_the_original_fifo_save_bindings() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            std::thread::sleep(std::time::Duration::from_millis(1100));
            let refresh = gui.preferences.begin_refresh_if_due().unwrap();
            drop(gui.change_preference(PreferenceChange::Worldview(Luke)));
            drop(gui.change_preference(PreferenceChange::Worldview(Suitcoiner)));
            assert_eq!(gui.worldview_sharing.queued.len(), 2);
            set_account(gui, 8);
            let receipt = super::super::GlobalSettings::load_or_initialize_preferences(
                &gui.config.coincube_directory,
            )
            .unwrap();
            drop(gui.update(Message::Preferences(refresh, Ok(receipt))));
            for _ in 0..2 {
                let save = gui.worldview_sharing.save.as_ref().unwrap().0.clone();
                finish_save(gui, save);
                assert!(gui.worldview_sharing.pending.is_none());
                assert!(gui.worldview_sharing.desired.is_none());
            }
            assert!(gui.worldview_sharing.queued.is_empty());
        });
    }

    #[test]
    fn closing_and_reopening_preferences_keeps_an_authorized_publication() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            let original = selected(gui);
            edit(gui, PreferenceChange::Worldview(Luke));
            let pane = gui.focus.unwrap();
            let tab = gui.panes.get(pane).unwrap().tabs[0].id;
            let open = |gui: &mut GUI| {
                drop(gui.update(Message::Pane(
                    pane,
                    super::super::pane::Message::tab(
                        tab,
                        super::super::tab::Message::OpenAppPreferences(
                            super::super::tab::PreferencesSource::Home,
                        ),
                    ),
                )))
            };
            let request = gui.worldview_sharing.pending;
            open(gui);
            assert_eq!(gui.worldview_sharing.pending, request);
            drop(gui.close_preferences_sheet());
            remote(gui, Ok(snapshot(Some(original))));
            assert!(matches!(
                gui.worldview_sharing.operation,
                Some(Request::Publish(_))
            ));
            let request = gui.worldview_sharing.pending;
            open(gui);
            assert_eq!(gui.worldview_sharing.pending, request);
            drop(gui.close_preferences_sheet());
            remote(gui, Ok(snapshot(Some(selected(gui)))));
            assert!(gui.worldview_sharing.pending.is_none());
            assert!(gui.preferences_sheet.is_none());
        });
    }

    #[test]
    fn navigation_during_save_and_network_updates_keeps_the_original_account() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            let original = selected(gui);
            drop(gui.change_preference(PreferenceChange::Worldview(Luke)));
            let save = gui.worldview_sharing.save.as_ref().unwrap().0.clone();
            let pane = gui.focus.unwrap();
            let home = super::super::home::Home::new_for_chain(
                gui.config.coincube_directory.clone(),
                None,
            )
            .0;
            gui.panes
                .get_mut(pane)
                .unwrap()
                .tabs
                .push(super::super::tab::Tab::new(
                    2,
                    super::super::tab::State::Home(home),
                ));
            gui.panes.get_mut(pane).unwrap().focused_tab = 1;
            assert!(gui.worldview_binding().is_none()); // The visible tab is signed out.
            finish_save(gui, save);
            remote(gui, Ok(snapshot(Some(original))));
            let luke = selected(gui);
            assert!(
                matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == luke)
            );
            remote(gui, Ok(snapshot(Some(luke))));
            assert!(gui.worldview_sharing.pending.is_none());
            gui.panes.get_mut(pane).unwrap().focused_tab = 0;
            edit(gui, PreferenceChange::Worldview(Suitcoiner));
            gui.panes.get_mut(pane).unwrap().focused_tab = 1;
            remote(gui, Ok(snapshot(Some(luke))));
            let suitcoiner = selected(gui);
            assert!(
                matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == suitcoiner)
            );
            remote(gui, Ok(snapshot(Some(suitcoiner))));
            assert!(gui.worldview_sharing.pending.is_none());
            assert_eq!(gui.worldview_sharing.binding.as_ref().unwrap().context.0, 7);
        });
    }

    #[test]
    fn edits_from_another_authenticated_origin_wait_for_the_in_flight_write() {
        for (failure, round_trip) in [
            (None, false),
            (None, true),
            (Some(Error::Unavailable), false),
            (Some(Error::Unavailable), true),
        ] {
            with_gui(move |gui| {
                use crate::app::settings::preferences::WorldviewMode::*;
                shared(gui);
                let original = selected(gui);
                edit(gui, PreferenceChange::Worldview(Luke));
                remote(gui, Ok(snapshot(Some(original))));
                let luke = selected(gui);
                let first = gui.worldview_sharing.in_flight.clone();
                another_home(gui, Some(7));
                edit(gui, PreferenceChange::Worldview(Suitcoiner));
                if round_trip {
                    let pane = gui.focus.unwrap();
                    gui.panes.get_mut(pane).unwrap().focused_tab = 0;
                    edit(gui, PreferenceChange::Worldview(Samurai));
                }
                assert_eq!(gui.worldview_sharing.in_flight, first);
                assert!(gui.worldview_sharing.pending.is_none());
                if let Some(error) = failure {
                    remote(gui, Err(error));
                    assert_eq!(gui.worldview_sharing.error, Some(error));
                    assert!(gui.worldview_sharing.desired.is_none());
                    assert!(gui.worldview_sharing.in_flight.is_none());
                    let pane = gui.focus.unwrap();
                    gui.panes.get_mut(pane).unwrap().focused_tab = usize::from(round_trip);
                    // The next accepted edit clears the pause and re-reads.
                    edit(gui, PreferenceChange::Worldview(Luke));
                    assert_eq!(gui.worldview_sharing.error, None);
                    assert!(matches!(
                        gui.worldview_sharing.operation,
                        Some(Request::Read)
                    ));
                } else {
                    remote(gui, Ok(snapshot(Some(luke))));
                    assert!(matches!(
                        gui.worldview_sharing.operation,
                        Some(Request::Read)
                    ));
                    remote(gui, Ok(snapshot(Some(luke))));
                    let latest = selected(gui);
                    assert!(
                        matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == latest)
                    );
                    remote(gui, Ok(snapshot(Some(latest))));
                    assert!(gui.worldview_sharing.in_flight.is_none());
                    assert_eq!(
                        gui.worldview_sharing.snapshot.as_ref().unwrap().selection,
                        Some(latest)
                    );
                }
            });
        }
    }

    #[test]
    fn unsuccessful_or_inactive_edits_in_another_origin_preserve_the_accepted_intent() {
        for kind in 0..3 {
            with_gui(move |gui| {
                use crate::app::settings::preferences::{Language, WorldviewMode::*};
                shared(gui);
                let original = selected(gui);
                edit(gui, PreferenceChange::Worldview(Luke));
                let binding = gui.worldview_sharing.binding.clone();
                let accepted = selected(gui);
                another_home(gui, Some(7));
                match kind {
                    0 => edit(gui, PreferenceChange::Worldview(Luke)), // No-op.
                    1 => edit_unrelated(gui, PreferenceChange::Language(Language::Spanish)),
                    _ => {
                        drop(gui.change_preference(PreferenceChange::Worldview(Suitcoiner)));
                        let request = gui.worldview_sharing.save.as_ref().unwrap().0.clone();
                        drop(gui.update(Message::Preferences(request, Err("disk full".into()))));
                    }
                }
                assert_eq!(gui.worldview_sharing.binding, binding);
                assert_eq!(gui.worldview_sharing.desired, Some(accepted));
                remote(gui, Ok(snapshot(Some(original))));
                assert!(
                    matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == accepted)
                );
                remote(gui, Ok(snapshot(Some(accepted))));
                assert!(gui.worldview_sharing.in_flight.is_none());
            });
        }
    }

    #[test]
    fn departed_origins_release_their_request_without_clearing_a_replacement() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            edit(gui, PreferenceChange::Worldview(Luke));
            let departed = gui.worldview_sharing.binding.clone().unwrap();
            let old = gui.worldview_sharing.pending.unwrap();
            set_account(gui, 8);
            drop(gui.worldview_complete(
                departed.origin.clone(),
                departed.context,
                old,
                Ok(snapshot(Some(selected(gui)))),
            ));
            assert!(gui.worldview_sharing.pending.is_none());
            assert!(gui.worldview_sharing.snapshot.is_none());
            set_account(gui, 7);
            edit(gui, PreferenceChange::Worldview(Suitcoiner));
            assert!(gui.worldview_sharing.pending.is_some());
            // A replacement binding owns its own request, even if an old
            // callback arrives after navigation/account replacement.
            set_account(gui, 8);
            edit(gui, PreferenceChange::Worldview(Samurai));
            let replacement = gui.worldview_sharing.pending;
            drop(gui.worldview_complete(
                departed.origin,
                departed.context,
                old,
                Err(Error::Locked),
            ));
            assert_eq!(gui.worldview_sharing.pending, replacement);
            assert_eq!(gui.worldview_sharing.error, None);
        });
    }

    #[test]
    fn enabled_account_is_discovered_after_restart_and_only_the_latest_queued_pair_is_sent() {
        with_gui(|gui| {
            use crate::app::settings::preferences::{ArtworkStyle, WorldviewMode::*};
            let original = selected(gui);
            std::thread::sleep(std::time::Duration::from_millis(1100));
            let refresh = gui.preferences.begin_refresh_if_due().unwrap();
            for change in [
                PreferenceChange::Worldview(Luke),
                PreferenceChange::Worldview(Suitcoiner),
                PreferenceChange::Worldview(Samurai),
                PreferenceChange::Worldview(Luke),
            ] {
                drop(gui.change_preference(change));
            }
            let receipt = super::super::GlobalSettings::load_or_initialize_preferences(
                &gui.config.coincube_directory,
            )
            .unwrap();
            drop(gui.update(Message::Preferences(refresh, Ok(receipt))));
            for index in 0..4 {
                let save = gui.worldview_sharing.save.as_ref().unwrap().0.clone();
                finish_save(gui, save);
                if index < 3 {
                    assert!(gui.worldview_sharing.pending.is_none());
                }
            }
            let expected = Selection {
                mode: Luke,
                artwork: ArtworkStyle::default_for(Luke),
            };
            assert_eq!(gui.worldview_sharing.desired, Some(expected));
            assert!(matches!(
                gui.worldview_sharing.operation,
                Some(Request::Read)
            ));
            remote(gui, Ok(snapshot(Some(original))));
            assert!(
                matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == expected)
            );
            remote(gui, Ok(snapshot(Some(expected))));
            assert!(gui.worldview_sharing.pending.is_none());
        });
    }

    fn sign_out(gui: &mut GUI) {
        let pane = gui.focus.unwrap();
        let super::super::tab::State::Home(home) =
            &mut gui.panes.get_mut(pane).unwrap().tabs[0].state
        else {
            panic!("home");
        };
        home.connect_account.user = None;
    }

    #[test]
    fn account_replacement_and_duplicate_callbacks_refuse_old_results() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            edit(gui, PreferenceChange::Worldview(Luke));
            let (binding, old) = gui.worldview_sharing.in_flight.clone().unwrap();
            set_account(gui, 8);
            drop(gui.worldview_complete(
                binding.origin.clone(),
                binding.context,
                old,
                Err(Error::Locked),
            ));
            assert_eq!(gui.worldview_sharing.error, None);
            assert!(gui.worldview_sharing.snapshot.is_none());
            assert!(gui.worldview_sharing.in_flight.is_none());
            // A duplicate callback finds nothing in flight and changes nothing.
            drop(gui.worldview_complete(
                binding.origin,
                binding.context,
                old,
                Err(Error::Conflict),
            ));
            assert_eq!(gui.worldview_sharing.error, None);
            assert!(gui.worldview_sharing.in_flight.is_none());
        });
    }

    #[test]
    fn opening_preferences_sends_no_request() {
        with_gui(|gui| {
            open_preferences(gui);
            assert!(gui.worldview_sharing.in_flight.is_none());
            assert!(gui.worldview_sharing.operation.is_none());
        });
    }

    #[test]
    fn publishes_each_accepted_worldview_for_a_signed_in_account_without_opt_in() {
        with_gui(|gui| {
            use crate::app::settings::preferences::{ArtworkStyle, WorldviewMode::*};
            // The account has never shared: the first edit still publishes.
            let mut previous = snapshot(None);
            for mode in [Luke, Samurai, Suitcoiner, Luke, Samurai] {
                edit(gui, PreferenceChange::Worldview(mode));
                // The wire keeps Mobile's artwork field: the mode's default.
                let expected = Selection {
                    mode,
                    artwork: ArtworkStyle::default_for(mode),
                };
                assert_eq!(gui.worldview_sharing.desired, Some(expected));
                assert!(matches!(
                    gui.worldview_sharing.operation,
                    Some(Request::Read)
                ));
                remote(gui, Ok(previous));
                assert!(
                    matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == expected)
                );
                remote(gui, Ok(snapshot(Some(expected))));
                assert!(gui.worldview_sharing.pending.is_none());
                assert!(gui.worldview_sharing.desired.is_none());
                previous = gui.worldview_sharing.snapshot.clone().unwrap();
                assert_eq!(previous.selection, Some(expected));
            }
        });
    }

    #[test]
    fn a_selection_cleared_elsewhere_is_replaced_by_the_next_edit() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            edit(gui, PreferenceChange::Worldview(Luke));
            remote(gui, Ok(snapshot(None)));
            let luke = selected(gui);
            assert!(
                matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == luke)
            );
        });
    }

    #[test]
    fn signed_out_noop_unrelated_and_failed_saves_never_publish() {
        with_gui(|gui| {
            use crate::app::settings::preferences::{Language, WorldviewMode::*};
            let current = gui.preferences.accepted().unwrap().preferences.worldview;
            edit(gui, PreferenceChange::Worldview(current)); // No-op.
            assert!(gui.worldview_sharing.in_flight.is_none());
            edit_unrelated(gui, PreferenceChange::Language(Language::Spanish));
            assert!(gui.worldview_sharing.in_flight.is_none());
            let other = if current == Luke { Samurai } else { Luke };
            drop(gui.change_preference(PreferenceChange::Worldview(other)));
            let request = gui.worldview_sharing.save.as_ref().unwrap().0.clone();
            drop(gui.update(Message::Preferences(request, Err("disk full".into()))));
            assert!(gui.worldview_sharing.in_flight.is_none());
            assert!(gui.worldview_sharing.desired.is_none());
            sign_out(gui);
            edit(gui, PreferenceChange::Worldview(other));
            assert!(gui.worldview_sharing.in_flight.is_none());
            assert!(gui.worldview_sharing.desired.is_none());
        });
    }

    #[test]
    fn a_save_that_lands_during_a_read_is_read_again_before_publication() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            let original = selected(gui);
            edit(gui, PreferenceChange::Worldview(Luke));
            drop(gui.change_preference(PreferenceChange::Worldview(Suitcoiner)));
            let save = gui.worldview_sharing.save.as_ref().unwrap().0.clone();
            remote(gui, Ok(snapshot(Some(original))));
            assert!(gui.worldview_sharing.in_flight.is_none());
            finish_save(gui, save);
            assert!(matches!(
                gui.worldview_sharing.operation,
                Some(Request::Read)
            ));
            remote(gui, Ok(snapshot(None)));
            let latest = selected(gui);
            assert!(
                matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == latest)
            );
        });
    }

    #[test]
    fn failures_pause_until_the_next_accepted_edit() {
        with_gui(|gui| {
            use crate::app::settings::preferences::WorldviewMode::*;
            shared(gui);
            for error in [Error::Conflict, Error::Unavailable, Error::Locked] {
                edit(gui, PreferenceChange::Worldview(Luke));
                remote(gui, Err(error));
                assert_eq!(gui.worldview_sharing.error, Some(error));
                assert!(gui.worldview_sharing.desired.is_none());
                assert!(gui.worldview_sharing.in_flight.is_none());
                edit(gui, PreferenceChange::Worldview(Suitcoiner));
                assert_eq!(gui.worldview_sharing.error, None);
                assert!(matches!(
                    gui.worldview_sharing.operation,
                    Some(Request::Read)
                ));
                remote(gui, Ok(snapshot(None)));
                let latest = selected(gui);
                assert!(
                    matches!(gui.worldview_sharing.operation, Some(Request::Publish(value)) if value == latest)
                );
                remote(gui, Ok(snapshot(Some(latest))));
                assert!(gui.worldview_sharing.pending.is_none());
            }
        });
    }
}
