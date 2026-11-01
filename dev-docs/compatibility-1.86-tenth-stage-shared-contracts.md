# Dev 1.86: tenth-stage shared framework contracts

## Evidence and scope

The Dev 1.85 device log embeds source
`84c7ca2b15962b7d73f8de6d33130e018ef2d6b3`, workflow `41307145898`, and
the unchanged universal bounded recorder. It is archived as
`outputs/device-logs/iDared-Dev-1.85-device-results.txt` (1,261,021 bytes,
SHA-256
`1d3d02da7e904eb21df2f8ff92b6f0d557db6eda272e1caca5b52d2aa5b4f928`).

This stage follows the first new exact blocker exposed for each title. It does
not alter the recorder, memory-fault policy, networking, unavailable resources,
orientation policy, or app-specific behavior.

## Findings

- **Blot**, **Dirt**, and **Spoing** remain regression passes in the user's
  tested scope.
- **Dark Nebula 2** completed a level, accepted a name, handled **No** at the
  score-submission prompt, progressed to the next level, and retained that
  progress after relaunch. It is now a **working checkpoint in the observed
  scope**, not proof that every level or unavailable online service works.
- **Crazy Golf** gameplay was already functional, but both standard volume
  sliders are absent and non-interactive. Six `UISlider` instances are created;
  the existing class explicitly does not render them and provides no tracking.
- **Cube** stops immediately at missing `_CGFontCreateWithFontName`. **Fragger**
  reaches its menu and level-pack selection, then reaches the same exact missing
  function. Their final symptom occurs at different stages but the blocker is
  shared.
- **Glowfish** now reaches a valid `UIPasteboard` and stops at missing
  `setData:forPasteboardType:`.
- **Glowfish HD** still reaches its menu and level selection, then fails to open
  the genuinely absent bundled `GameData/Levels/Level_01.gfl` and deliberately
  aborts. Its portrait keyboard in a landscape game remains a separate
  orientation defect. No resource or success response is fabricated here.
- **Hamlet** remains playable after its unchanged initial rotated/cropped
  presentation. It is not modified in this stage.
- **Junny** passes the Dev 1.85 run-loop cancellation contract and next stops
  during valid `UITableViewCell` class initialization.
- **Parachute Ninja** remains at the same invalid app-side dispatch: a real
  `UIApplication` receives `domain`. It is deliberately unchanged; adding
  unrelated error selectors to UIApplication would hide the invalid origin.
- **Rolando** passes the run-loop cancellation contract and next sends the
  standard inherited `initWithPath:` initializer to its custom `NSBundle`
  subclass.
- **Toy Bot** and **Toy Bot 2** pass their slider flag and next share missing
  `UITextView.setKeyboardType:` with the same valid argument.

## Implementation boundary

- Core Graphics now exports `CGFontCreateWithFontName`, resolving historical
  system aliases and registered bundled fonts through the existing font
  machinery. It creates an independently owned `CGFont` and returns null for an
  unknown or null name.
- `UIPasteboard` stores retained typed `NSData` values by type, supports typed
  retrieval, increments `changeCount`, and releases those values at teardown.
- `UITextView` stores and reports `keyboardType`; actual host keyboard layout
  remains governed by the existing input bridge.
- `NSBundle` gains the standard `initWithPath:` initializer, including main
  bundle identity, Info.plist validation, retained state, and subclass-safe
  initialization.
- A partial historical `UITableViewCell` contract supplies stable standard
  subviews, reuse identity, selection/highlight state, accessory/background
  ownership, old text/image accessors, and bounded default layout. Table styling
  and accessory artwork remain partial.
- `UISlider` now has value/range/continuous state, retained state-specific
  artwork, visible fallback track/thumb views, layout, touch tracking, and
  correct continuous or end-only value-change delivery. Animation remains
  immediate.

Focused tests cover font-name construction, pasteboard typed-data ownership,
table-cell identity/state/subviews, and slider state/layout. The implementation
is general and contains no bundle-identifier checks.

## Device predictions

- Crazy Golf's two volume sliders should render and respond.
- Cube and Fragger should pass their exact font-construction blocker; this may
  expose a later blocker rather than establish gameplay.
- Glowfish, Junny, Rolando, Toy Bot, and Toy Bot 2 should each pass the exact
  contract recorded above and either advance or expose a later distinct issue.
- Glowfish HD's missing level file, its keyboard orientation, Hamlet's opening
  orientation, and Parachute Ninja's invalid callback origin are not claimed as
  fixed.
- Blot, Dark Nebula 2, Dirt, and Spoing are focused regressions. No save reset is
  requested.

## Verified delivery

- Final implementation commit:
  `9b9e2df984d7390b2e39ca90e55fc3f8e59e2696`.
- [Hosted tests 41313333552](https://github.com/bookieart1/iDared32bit/actions/runs/41313333552):
  **145/145 development-library tests passed on macOS and 145/145 on
  Windows**. The native Foundation reference and both bounded recorder
  reproducers passed. Full guest-executable tests were skipped on the
  development branch. The separately allowed-to-fail macOS, Windows and
  Android lint stages each emitted exit code 1; overall workflow success is
  therefore not a clean-lint claim.
- [IPA build 41313333541](https://github.com/bookieart1/iDared32bit/actions/runs/41313333541):
  arm64 compilation and unsigned packaging completed successfully. Artifact ID
  `13086208966`; ZIP SHA-256
  `ba577815a30bb7cd19697969866a4743140ee7540b243067b0db7890ae35cf40`,
  matching GitHub's published digest.
- Local `outputs/iDared-Dev-1.86.ipa`, 20,003,299 bytes, SHA-256
  `fd736c918b714499ad4889001ff51a36c5dc91c1447ed17faf583c4684077909`,
  passed arm64, development identity, unsigned/no-provisioning, embedded
  commit/workflow, universal-recorder and new-contract marker checks.

Verification: `outputs/iDared-Dev-1.86.verification.json`. Device results remain
pending; passing a recorded missing contract is not yet a gameplay claim.
