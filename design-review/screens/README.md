# Exported screens

PNG renders of the Desktop canvas at version 345 (artifact version
`1795761319-02e2`, exported 2026-11-27; the `H-*`, `G-*` and `O-*` study
sources are still from v175; the Launcher backdrop renders are v175 images), so the design can be read without a
claude.ai session. Pixels only: exact copy, spacing, tokens and state logic are
in `../canvas/<source>`.

- `dark/` is the "Prototype (clickable)" canvas page with the device settings at
  their defaults: Appearance Dark, Developer mode off, Bitcoin Blake2b off,
  Launcher style Cubes.
- `light/` is the "Light theme" canvas page: the same screens mounted with
  `force-theme=light`, named after the dark board they mount, plus the
  Launcher's List view (`P-Launcher-List.png`).
- `studies/` holds the v175 study pages (Launcher backdrop, page header, ghost
  Cubes, opening on the Cube). Their decisions are folded into the prototype; the
  studies stay for the reasoning and the states they show.
- Each image is the board's default state at its canvas size, not scaled
  (mostly 1440 × 900; the canvas sets each size). Other states are levers in the source
  (`data-props`). Animated boards (`P-Opening`, the `O-*` finale and glow
  boards) are caught 2.5 s after they mount; the motion itself is only in the
  live canvas.
- All data is synthetic.

## How these were made

`render/` renders the board sources locally, without the Claude Design runtime:
`render/support.js` is a small stand-in for the runtime's template features
(`{{ holes }}`, `sc-if`, `sc-for`, `dc-import`, `DCLogic` state), and
`render/_blob/` holds the three images the boards reference. To regenerate after
the canvas sources change:

```bash
NODE_PATH=<dir containing playwright-core> node design-review/screens/render/capture.js
```

It needs Google Chrome (or `CHROME=<path>`). Pass board names to render only
those. The stand-in ignores event handlers, so it shows default states only. If a
render disagrees with the live canvas, the live canvas wins.

## Prototype, dark

| Image | Board | Source |
| --- | --- | --- |
| `dark/P-Launcher.png` | 1 · Launcher (filters, plan, Cube details face → PIN/passkey → opening) | `P-Launcher.dc.html` |
| `dark/P-Unlock.png` | PIN / passkey unlock face (error states) | `P-Unlock.dc.html` |
| `dark/P-Opening.png` | Opening Cube (after checks / create / restore) | `P-Opening.dc.html` |
| `dark/P-Locking.png` | Locking Cube (hand-off → Launcher) | `P-Locking.dc.html` |
| `dark/P-AppPrefs.png` | App preferences (Appearance, Launcher, Balances & motion, Developer, Diagnostics, About) | `P-AppPrefs.dc.html` |
| `dark/P-Connect.png` | Connect sheet (Account, Plan & billing, Contacts, Devices & security, Recovery services) | `P-Connect.dc.html` |
| `dark/P-SignIn.png` | Connect sign-in / sign-up | `P-SignIn.dc.html` |
| `dark/P-Create.png` | Create Cube (Launcher sheet: Cube, Backup, Done) | `P-Create.dc.html` |
| `dark/P-Restore.png` | Import or restore (Launcher sheet) | `P-Restore.dc.html` |
| `dark/P-GapsInstaller.png` | Create Vault wizard (inside the Cube, from "Set up Vault") | `P-GapsInstaller.dc.html` |
| `dark/P-BorderWallet.png` | Border Wallet | `P-BorderWallet.dc.html` |
| `dark/P-Delete.png` | Delete Cube (Launcher modal) | `P-Delete.dc.html` |
| `dark/P-Overview.png` | 2 · Overview (and the Spark and Vault wallet pages) | `P-Overview.dc.html` |
| `dark/P-Activity.png` | Activity (sidebar) | `P-Activity.dc.html` |
| `dark/P-Send.png` | Send (You send → They receive; Spark and Vault) | `P-Send.dc.html` |
| `dark/P-Receive.png` | Receive (They send → You receive; Spark and Vault) | `P-Receive.dc.html` |
| `dark/P-Transfer.png` | Transfer (Cube-level: between wallets and into other Cubes) | `P-Transfer.dc.html` |
| `dark/P-Swap.png` | Swap BTC ⇄ USDB (from the Spark page) · Details, Review, Swapped | `P-Swap.dc.html` |
| `dark/P-Advanced.png` | Vault tools (Coin control, PSBT proposals, Recovery path, Split coins) | `P-Advanced.dc.html` |
| `dark/P-Split.png` | Bitcoin Blake2b copy (explainer and steps) | `P-Split.dc.html` |
| `dark/P-Advanced-B2B.png` | Blake2b copy › Tools · Coin control and Split coins from Bitcoin Blake2b | `P-Advanced-B2B.dc.html` |
| `dark/P-Signing.png` | Signing & phone pairing (PSBT signers, pairing QR and code) | `P-Signing.dc.html` |
| `dark/P-Settings.png` | Settings (Cube / Vault / Spark scopes, tabs; from the sidebar foot) | `P-Settings.dc.html` |
| `dark/P-Members.png` | Members (Cube Settings tab, behind a build flag, off by default) | `P-Members.dc.html` |
| `dark/P-Duress.png` | Duress (Connect sheet, gated by plan and server flag) | `P-Duress.dc.html` |
| `dark/P-GapsAccount.png` | Settings sub-flows (Recovery Kit, Recovery alerts) | `P-GapsAccount.dc.html` |
| `dark/P-BuySell.png` | Marketplace · Buy/Sell | `P-BuySell.dc.html` |
| `dark/P-P2P.png` | Marketplace · P2P | `P-P2P.dc.html` |
| `dark/P-AI-Access.png` | AI access · paired clients, pairing, permissions, revoke (from Settings) | `P-AI-Access.dc.html` |
| `dark/P-AI-Setup.png` | Wallet proposal (planned · build feature, AI access) · AI-led wallet setup | `P-AI-Setup.dc.html` |
| `dark/P-AI-Transaction.png` | Prepared transaction (planned · build feature, AI access) · draft, review, signing | `P-AI-Transaction.dc.html` |
| `dark/P-AppPrefs-InCube.png` | App preferences · over an open Cube (from the Cube top bar) | `P-AppPrefs-InCube.dc.html` |
| `dark/P-Connect-InCube.png` | Connect · over an open Cube (from the Cube top bar) | `P-Connect-InCube.dc.html` |
| `dark/P-Spark.png` | Spark wallet page (Overview board on page=spark; link target) | `P-Spark.dc.html` |
| `dark/P-Vault.png` | Vault wallet page (Overview board on page=vault; link target) | `P-Vault.dc.html` |

## Light theme

| Image | Board | Source |
| --- | --- | --- |
| `light/P-Launcher.png` | Launcher · light · Cubes | `P-Launcher-Light.dc.html` |
| `light/P-Launcher-List.png` | Launcher · light · List | `P-Launcher-Light-List.dc.html` |
| `light/P-Unlock.png` | PIN / passkey unlock face (error states) · light | `P-Light-Unlock.dc.html` |
| `light/P-Opening.png` | Opening Cube (after checks / create / restore) · light | `P-Light-Opening.dc.html` |
| `light/P-AppPrefs.png` | App preferences (Appearance, Launcher, Balances & motion, Developer, Diagnostics, About) · light | `P-Light-AppPrefs.dc.html` |
| `light/P-Connect.png` | Connect sheet (Account, Plan & billing, Contacts, Devices & security, Recovery services) · light | `P-Light-Connect.dc.html` |
| `light/P-SignIn.png` | Connect sign-in / sign-up · light | `P-Light-SignIn.dc.html` |
| `light/P-Create.png` | Create Cube (Launcher sheet: Cube, Backup, Done) · light | `P-Light-Create.dc.html` |
| `light/P-Restore.png` | Import or restore (Launcher sheet) · light | `P-Light-Restore.dc.html` |
| `light/P-GapsInstaller.png` | Create Vault wizard (inside the Cube, from "Set up Vault") · light | `P-Light-GapsInstaller.dc.html` |
| `light/P-BorderWallet.png` | Border Wallet · light | `P-Light-BorderWallet.dc.html` |
| `light/P-Delete.png` | Delete Cube (Launcher modal) · light | `P-Light-Delete.dc.html` |
| `light/P-Overview.png` | 2 · Overview (and the Spark and Vault wallet pages) · light | `P-Light-Overview.dc.html` |
| `light/P-Activity.png` | Activity (sidebar) · light | `P-Light-Activity.dc.html` |
| `light/P-Send.png` | Send (You send → They receive; Spark and Vault) · light | `P-Light-Send.dc.html` |
| `light/P-Receive.png` | Receive (They send → You receive; Spark and Vault) · light | `P-Light-Receive.dc.html` |
| `light/P-Transfer.png` | Transfer (Cube-level: between wallets and into other Cubes) · light | `P-Light-Transfer.dc.html` |
| `light/P-Swap.png` | Swap BTC ⇄ USDB (from the Spark page) · Details, Review, Swapped · light | `P-Light-Swap.dc.html` |
| `light/P-Advanced.png` | Vault tools (Coin control, PSBT proposals, Recovery path, Split coins) · light | `P-Light-Advanced.dc.html` |
| `light/P-Split.png` | Bitcoin Blake2b copy (explainer and steps) · light | `P-Light-Split.dc.html` |
| `light/P-Signing.png` | Signing & phone pairing (PSBT signers, pairing QR and code) · light | `P-Light-Signing.dc.html` |
| `light/P-Settings.png` | Settings (Cube / Vault / Spark scopes, tabs; from the sidebar foot) · light | `P-Light-Settings.dc.html` |
| `light/P-Members.png` | Members (Cube Settings tab, behind a build flag, off by default) · light | `P-Light-Members.dc.html` |
| `light/P-Duress.png` | Duress (Connect sheet, gated by plan and server flag) · light | `P-Light-Duress.dc.html` |
| `light/P-GapsAccount.png` | Settings sub-flows (Recovery Kit, Recovery alerts) · light | `P-Light-GapsAccount.dc.html` |
| `light/P-BuySell.png` | Marketplace · Buy/Sell · light | `P-Light-BuySell.dc.html` |
| `light/P-P2P.png` | Marketplace · P2P · light | `P-Light-P2P.dc.html` |
| `light/P-AI-Access.png` | AI access · paired clients, pairing, permissions, revoke (from Settings) · light | `P-Light-AI-Access.dc.html` |
| `light/P-AI-Setup.png` | Wallet proposal (planned · build feature, AI access) · AI-led wallet setup · light | `P-Light-AI-Setup.dc.html` |
| `light/P-AI-Transaction.png` | Prepared transaction (planned · build feature, AI access) · draft, review, signing · light | `P-Light-AI-Transaction.dc.html` |

## Studies

| Image | Board | Canvas page | Source |
| --- | --- | --- | --- |
| `studies/W-Create.png` | Create Cube · modal over the Launcher, Close top right (Play to walk it) | Create & Import · modal wizards (proposal) | `W-Create.dc.html` |
| `studies/W-Create-Cube.png` | Create Cube · step 2 (Bitcoin Blake2b) | Create & Import · modal wizards (proposal) | `W-Create-Cube.dc.html` |
| `studies/W-LauncherBar.png` | Launcher bottom bar · before, after, and plan full | Create & Import · modal wizards (proposal) | `W-LauncherBar.dc.html` |
| `studies/W-Import.png` | Overview · clickable: walk every option in Play | Create & Import · modal wizards (proposal) | `W-Import.dc.html` |
| `studies/W-Import-Signet.png` | Overview · signed out on Signet: Connect options wait for sign-in, wallet import not on this network | Create & Import · modal wizards (proposal) | `W-Import-Signet.dc.html` |
| `studies/W-Import-Menu.png` | Network menu · the same menu in Create and Import | Create & Import · modal wizards (proposal) | `W-Import-Menu.dc.html` |
| `studies/W-Import-Kit.png` | Recovery Kit 1 · Cube: filtered to the network (Signet), Show all to switch | Create & Import · modal wizards (proposal) | `W-Import-Kit.dc.html` |
| `studies/W-Import-Wallet.png` | Import Bitcoin wallet 1 · name the Cube (Bitcoin and Bitcoin Blake2b only) | Create & Import · modal wizards (proposal) | `W-Import-Wallet.dc.html` |
| `studies/W-Import-Phone.png` | Recover with phone 1 · Cubes whose backup is on a paired phone | Create & Import · modal wizards (proposal) | `W-Import-Phone.dc.html` |
| `studies/W-Import-Phone-Approve.png` | Recover with phone 2 · waiting for approval on the phone | Create & Import · modal wizards (proposal) | `W-Import-Phone-Approve.dc.html` |
| `studies/W-Import-Overview-In.png` | Overview · signed in on Bitcoin: Connect tags turn green | Create & Import · modal wizards (proposal) | `W-Import-Overview-In.dc.html` |
| `studies/W-Import-Choice.png` | Two backups · opening a Connect Cube that has both a Recovery Kit and a phone backup | Create & Import · modal wizards (proposal) | `W-Import-Choice.dc.html` |
| `studies/W-Import-Kit-Password.png` | Recovery Kit 2 · recovery password | Create & Import · modal wizards (proposal) | `W-Import-Kit-Password.dc.html` |
| `studies/W-Import-Kit-WrongPw.png` | Recovery Kit 2 · wrong password | Create & Import · modal wizards (proposal) | `W-Import-Kit-WrongPw.dc.html` |
| `studies/W-Import-Kit-Pin.png` | Recovery Kit 3 · set a PIN (no passkey on restore) and the Bitcoin server | Create & Import · modal wizards (proposal) | `W-Import-Kit-Pin.dc.html` |
| `studies/W-Import-Kit-None.png` | Recovery Kit · no Recovery Kits on this network (Regtest) | Create & Import · modal wizards (proposal) | `W-Import-Kit-None.dc.html` |
| `studies/W-Import-Phone-Denied.png` | Recover with phone 2 · declined on the phone | Create & Import · modal wizards (proposal) | `W-Import-Phone-Denied.dc.html` |
| `studies/W-Import-Phone-Expired.png` | Recover with phone 2 · request expired | Create & Import · modal wizards (proposal) | `W-Import-Phone-Expired.dc.html` |
| `studies/W-Import-Phone-None.png` | Recover with phone · no phone backups on this network (Testnet4) | Create & Import · modal wizards (proposal) | `W-Import-Phone-None.dc.html` |
| `studies/W-Import-Wallet-Source.png` | Import Bitcoin wallet 2 · source and Bitcoin server | Create & Import · modal wizards (proposal) | `W-Import-Wallet-Source.dc.html` |
| `studies/W-Import-Wallet-Review.png` | Import Bitcoin wallet 4 · review | Create & Import · modal wizards (proposal) | `W-Import-Wallet-Review.dc.html` |
| `studies/W-Import-Wallet-Scan.png` | Import Bitcoin wallet 5 · scanning history | Create & Import · modal wizards (proposal) | `W-Import-Wallet-Scan.dc.html` |
| `studies/W-Import-Wallet-ScanError.png` | Import Bitcoin wallet 5 · scan failed: retry or change server | Create & Import · modal wizards (proposal) | `W-Import-Wallet-ScanError.dc.html` |
| `studies/W-Import-Wallet-Pin.png` | Import Bitcoin wallet 6 · set a PIN | Create & Import · modal wizards (proposal) | `W-Import-Wallet-Pin.dc.html` |
| `studies/W-Import-Wallet-Done.png` | Import Bitcoin wallet 7 · Done | Create & Import · modal wizards (proposal) | `W-Import-Wallet-Done.dc.html` |
| `studies/W-Import-Kit-Done.png` | Recovery Kit 4 · Done | Create & Import · modal wizards (proposal) | `W-Import-Kit-Done.dc.html` |
| `studies/W-Import-Phone-Done.png` | Recover with phone 4 · Done | Create & Import · modal wizards (proposal) | `W-Import-Phone-Done.dc.html` |
| `studies/W-Create-Done.png` | Create Cube · Done | Create & Import · modal wizards (proposal) | `W-Create-Done.dc.html` |
| `studies/W-Create-Backup.png` | Backup · choose Seed phrase and/or Recovery Kit | Create & Import · modal wizards (proposal) | `W-Create-Backup.dc.html` |
| `studies/W-Create-Seed-Intro.png` | Seed phrase 1 of 3 · before you see the words | Create & Import · modal wizards (proposal) | `W-Create-Seed-Intro.dc.html` |
| `studies/W-Create-Seed-Words.png` | Seed phrase 2 of 3 · the 12 words, hidden until Reveal | Create & Import · modal wizards (proposal) | `W-Create-Seed-Words.dc.html` |
| `studies/W-Create-Seed-Verify.png` | Seed phrase 3 of 3 · check words 3, 7 and 11 | Create & Import · modal wizards (proposal) | `W-Create-Seed-Verify.dc.html` |
| `studies/W-Create-Kit-Password.png` | Recovery Kit 1 of 2 · recovery password, checked live | Create & Import · modal wizards (proposal) | `W-Create-Kit-Password.dc.html` |
| `studies/W-Create-Kit-Where.png` | Recovery Kit 2 of 2 · Connect and/or a file on this computer | Create & Import · modal wizards (proposal) | `W-Create-Kit-Where.dc.html` |
| `studies/W-Create-Backup-Done.png` | Backup · seed phrase confirmed, Continue enabled | Create & Import · modal wizards (proposal) | `W-Create-Backup-Done.dc.html` |
| `studies/MV-Overview-Before.png` | Before · Overview today, with the worldview art the app draws beside Total balance | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Overview-Before.dc.html` |
| `studies/MV-Overview-After.png` | After · Receive and Send as on Mobile, no artwork, fill instead of outlines | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Overview-After.dc.html` |
| `studies/MV-Overview-After-Light.png` | After · light theme | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Overview-After-Light.dc.html` |
| `studies/MV-Cube-Sidebar.png` | Cube in the sidebar · above the selector, four faces; click the sidebar or a face | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Cube-Sidebar.dc.html` |
| `studies/MV-Notices-Today.png` | Notices and status · today in the app | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Notices-Today.dc.html` |
| `studies/MV-Notices-Proposed.png` | Notices and status · proposed | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Notices-Proposed.dc.html` |
| `studies/MV-Receive-Today.png` | Receive · today in the app | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Receive-Today.dc.html` |
| `studies/MV-Receive-Proposed.png` | Receive · proposed · They send open | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Receive-Proposed.dc.html` |
| `studies/MV-Send-Today.png` | Send · today in the app | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Send-Today.dc.html` |
| `studies/MV-Send-Proposed.png` | Send · proposed · They receive open | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Send-Proposed.dc.html` |
| `studies/MV-Receive-Proposed-YouReceive.png` | Receive · proposed · You receive open (Bitcoin or US Dollar, in Spark) | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Receive-Proposed-YouReceive.dc.html` |
| `studies/MV-Send-Proposed-YouSend.png` | Send · proposed · You send open (Bitcoin or US Dollar, in Spark) | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Send-Proposed-YouSend.dc.html` |
| `studies/MV-Type-Scale.png` | Type scale · Mobile V3, Desktop today, Desktop proposed | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Type-Scale.dc.html` |
| `studies/MV-Type-Today.png` | Type · Overview today | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Type-Today.dc.html` |
| `studies/MV-Type-Proposed.png` | Type · Overview on Mobile V3's steps | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Type-Proposed.dc.html` |
| `studies/MV-Cube-Faces.png` | Cube in the sidebar · faces, states and unlock | Overview · closer to Mobile V3 (study, Nov 2026) | `MV-Cube-Faces.dc.html` |
| `studies/D4-Notices.png` | Standing notices · dark | D4 review · standing notices (Nov 2026) | `D4-Notices.dc.html` |
| `studies/D4-Notices-Light.png` | Standing notices · light | D4 review · standing notices (Nov 2026) | `D4-Notices-Light.dc.html` |
| `studies/H-Overview.png` | Overview · shared page head | Page header study | `H-Overview.dc.html` |
| `studies/H-Wallet.png` | Wallet (Spark) · one Swap BTC ⇄ USDB in the head; rows keep Send and Receive; See all opens Activity filtered, Wallet settings opens Settings scoped | Page header study | `H-Wallet.dc.html` |
| `studies/H-Settings.png` | Settings · scope in the head, tabs on the rule | Page header study | `H-Settings.dc.html` |
| `studies/H-Send.png` | Send · the shipped Payment details card under the new head: Back on the right, steps on the rule | Page header study | `H-Send.dc.html` |
| `studies/H-Send-From.png` | Send · You send Change: pay from Bitcoin or US Dollar (USDB) | Page header study | `H-Send-From.dc.html` |
| `studies/H-Send-To.png` | Send · They receive Change: Lightning, on-chain, Spark, or a cross-chain stablecoin | Page header study | `H-Send-To.dc.html` |
| `studies/H-Receive.png` | Receive · same head (Vault shown) | Page header study | `H-Receive.dc.html` |
| `studies/H-Vault.png` | Wallet (Vault) · same page as Spark: one Bitcoin row, protection strip (time lock, PSBTs), Tools and Wallet settings in the head | Page header study | `H-Vault.dc.html` |
| `studies/H-Send-Vault.png` | Send (Vault) · same card as Spark, plus network fee and coins; Sign step for 2 of 3 | Page header study | `H-Send-Vault.dc.html` |
| `studies/H-Settings-Vault.png` | Settings scoped to Vault · spending keys and recovery path | Page header study | `H-Settings-Vault.dc.html` |
| `studies/H-Convert.png` | Swap BTC ⇄ USDB · from the Spark page head: From and To swap, amount, indicative rate; Get quote | Page header study | `H-Convert.dc.html` |
| `studies/H-Convert-Review.png` | Swap · review: quoted rate, fee, max price change, then Swap | Page header study | `H-Convert-Review.dc.html` |
| `studies/H-Settings-Spark.png` | Settings scoped to Spark · auto-convert received bitcoin, pay from USDB, max price change | Page header study | `H-Settings-Spark.dc.html` |
| `studies/H-Settings-Spark-Off.png` | Settings · Spark · turning auto-convert off converts the USDB balance back | Page header study | `H-Settings-Spark-Off.dc.html` |
| `studies/G-Launcher.png` | Launcher · Connect-only Cubes as ghosts, Key open | Connect Cubes as ghosts | `G-Launcher.dc.html` |
| `studies/G-Ghost-Details.png` | Ghost selected · the face each variety turns to | Connect Cubes as ghosts | `G-Ghost-Details.dc.html` |
| `studies/O-Launcher.png` | Launcher · PIN or passkey, then the Cube tumbles to its loading face and opens | Opening on the Cube | `O-Launcher.dc.html` |
| `studies/O-Finale-A.png` | Finale A · Bloom | Opening on the Cube | `O-Finale-A.dc.html` |
| `studies/O-Finale-B.png` | Finale B · Flash | Opening on the Cube | `O-Finale-B.dc.html` |
| `studies/O-Finale-C.png` | Finale C · Zoom | Opening on the Cube | `O-Finale-C.dc.html` |
| `studies/O-Glow-A.png` | Edge glow A · Soft | Opening on the Cube | `O-Glow-A.dc.html` |
| `studies/O-Glow-B.png` | Edge glow B · Bright | Opening on the Cube | `O-Glow-B.dc.html` |
| `studies/O-List.png` | List view · full screen | Opening on the Cube | `O-List.dc.html` |
| `studies/O-Stall.png` | Stalled loader · Cancel | Opening on the Cube | `O-Stall.dc.html` |
| `studies/P-AppPrefs-Rail.png` | B · App preferences › Launcher · Network activity on/off and source | Launcher backdrop (design B) | `P-AppPrefs-Rail.dc.html` |
| `studies/P-Launcher-Rail-Light.png` | B · Launcher · light | Launcher backdrop (design B) | `P-Launcher-Rail-Light.dc.html` |
| `studies/P-Launcher-Rail-Node.png` | B · Launcher · network activity from a local node already running | Launcher backdrop (design B) | `P-Launcher-Rail-Node.dc.html` |
| `studies/P-Launcher-Rail.png` | B · Launcher · network activity via COINCUBE \| Connect (no local node running) | Launcher backdrop (design B) | `P-Launcher-Rail.dc.html` |
