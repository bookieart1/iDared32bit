# Dev 1.81: fifth-stage contracts and app-picker continuity

## Evidence and scope

Three unmodified Dev 1.80 logs are preserved outside the repository:

- `outputs/device-logs/iDared-Dev-1.80-device-results-part-1.txt` — 518,004
  bytes, SHA-256
  `fe71d40fe990738bd292d5a6caf4ecdf73af87bbabe266354b5e025ca1c742aa`.
- `outputs/device-logs/iDared-Dev-1.80-device-results-part-2.txt` — 233,125
  bytes, SHA-256
  `2837b1f5df297486bfd0448713a168b03f8f61dbffc5448fca3ceab841594b4f`.
- `outputs/device-logs/iDared-Dev-1.80-glowfish-hd-profile-crash.txt` —
  452,462 bytes, SHA-256
  `f726145e41b511cc0eda5540ca4550d83e109c786efa8eca3b997f2ba790e36f`.

All embed source `764d60e`, workflow `41292778118`, and the universal bounded
flight recorder. This iteration does not alter that recorder, reset saves,
weaken pointer/object checks, or add application-identifier branches.

| App | Dev 1.80 decisive evidence | Dev 1.81 action and boundary |
|---|---|---|
| Dark Nebula 2 | Menu and loading run for 6.319 seconds, 251 main loops and 127 EAGL returns before missing `_object_setInstanceVariable`. | Implement the historical raw runtime ivar store over the same stable metadata as the getter. It returns the `Ivar`, does no ownership operation, and handles null/missing inputs. |
| Glowfish | Core Motion initialization and resource loading pass; startup reaches missing `_dispatch_once` after 1.465 seconds. | Implement the 32-bit zero-initialized once predicate and synchronous block execution, including a bounded cooperative wait while another guest thread owns initialization. Full GCD queue scheduling remains out of scope. |
| Glowfish HD | Core Motion passes and the menu/profile-name UI runs. The reproducible post-name transition presents 892 frames over 19.966 seconds, then calls missing `+[NSDictionary dictionaryWithObjects:forKeys:count:]`. | Implement the standard pointer-array constructor for immutable and mutable concrete dictionaries with copied keys, retained values, empty-count handling and existing strict nil checks. |
| Hamlet | The app runs for 33.823 seconds, 4,072 main loops and 1,943 EAGL returns, then exits cleanly through Home. The user observed a transient wrong initial layout before landscape corrected itself. | Working checkpoint in the observed interaction. Record the orientation defect; do not mix an unproven rotation change into this runtime batch. |
| Parachute Ninja | Three genuine offline request failures return. A later main-thread selector callback passes `UIApplication` where guest code treats its argument as an error and sends `domain`; the strict missing-selector panic follows after 140 ms. | No compatibility change. Adding `domain` to `UIApplication` would conceal an invalid-object origin. A future bounded callback/selector-origin investigation must identify why that object occupies the error argument. |
| Blot / Dirt / Spoing | The first two return normally through Home and Spoing remains a clean working control. A later Dirt test clarified that its physical-rotation gravity mechanic is inert and a held touch can reverse when it leaves the old app viewport for a modern phone's black bar. | Blot and Spoing remain regression checkpoints. Dirt receives two general input-contract corrections described below. |

Glowfish HD's native keyboard appeared portrait-oriented inside the landscape
game but accepted text. That is a separate UIKit/host-input orientation defect;
the terminal crash is the missing Foundation constructor above.

## App-picker continuity

The native picker previously rendered only the selected page and changed its
page index after a completed swipe, producing a hard cut. Dev 1.81 keeps the
current and adjacent pages mounted, follows the horizontal drag, eases into the
destination, and applies restrained resistance at the first/last page.

The picker now stores its page in the development app's `UserDefaults`. Rust's
reload page is nullable, so an explicit in-session reload still wins while a
fresh picker invocation restores the saved page. The page is clamped to the
current app count. This preserves page position after normal guest Home return,
an emulator-caught panic, or a complete iDared process restart without touching
guest saves or the separate App Store installation.

## Physical orientation and viewport-bound touch input

Dirt's previously tested traversal did not exercise two important mechanics,
so its earlier “working checkpoint” is narrowed to **partial** rather than
silently treated as whole-game compatibility:

- Rotating the physical device is supposed to change gravity, but Dev 1.80
  leaves it unchanged. The prior Dirt launch never activated `UIAccelerometer`
  or Core Motion. The separate `UIDevice` implementation only returned the
  fixed interface orientation and posted a seeded launch notification, even
  though Apple's historical contract says
  `beginGeneratingDeviceOrientationNotifications` enables accelerometer-based
  updates to the physical `orientation` property and change notifications.
- A held movement touch can leave the 3:2 guest viewport and enter a modern
  iPhone's surrounding black bar. The common input transform forwarded that
  impossible negative/over-width co-ordinate to the guest, where Dirt could
  interpret it as the opposite side.

Dev 1.81 samples the existing real/simulated gravity source at 20 Hz only while
an app has requested physical-orientation notifications. It distinguishes
portrait, landscape and face-up/down using the dominant gravity axis, rejects
weak/non-finite readings, requires a changed reading to remain stable for 100
ms, supports nested begin/end requests, and posts the existing historical
notification. This does not rotate the interface or fabricate motion.

Physical touch input is now clamped to the visible guest viewport before its
existing scale/rotation transform, and the resulting guest co-ordinate is kept
inside the original touchscreen bounds. A drag that enters a black bar remains
at the nearest original-device edge. Controller-generated virtual touch points
remain independent of the host viewport.

Both corrections are application-independent. Focused unit coverage checks
gravity-axis classification, rejection of invalid/weak readings, and black-bar
clamping on every side. Actual sensor direction, notification delivery and
Dirt's gameplay response still require the iPhone test.

## Validation boundary and prediction

Focused guest coverage exercises raw ivar assignment, once-only block
execution, and the pointer-array dictionary constructor. The picker itself must
be compiled by the iOS workflow and checked on a device. Hosted library tests,
guest executable tests, iOS compilation, package verification and physical
device behavior remain separate claims.

Predictions:

- Dark Nebula 2, Glowfish and Glowfish HD pass their exact recorded blockers
  and expose either later contracts or a usable next stage.
- Hamlet remains usable in the tested scope; its transient orientation and
  Glowfish HD's sideways keyboard may remain.
- Parachute Ninja fails at the same strict invalid-object boundary because it
  is intentionally not masked.
- Picker pages move visibly during a swipe and reopen on the last-used page.
- Dirt responds to stable physical rotations by changing its gravity and a
  held edge touch no longer reverses after entering a black bar.
- Blot and Spoing retain their stated behavior.

## Verified delivery

- Final implementation commit:
  `178cc5197ebc80415538d12aaa6458d3a3e47abc`. The fifth-stage runtime and
  picker work is in parent commit `bb95d1e0147150f0784f1d0b77f3c59184adf0f4`;
  the final commit adds the two general Dirt input contracts.
- [Hosted tests 41294836579](https://github.com/bookieart1/iDared32bit/actions/runs/41294836579)
  completed successfully at the job level. macOS x86_64 and Windows each ran
  **132/132 passing library tests**; the two bounded diagnostic reproducers
  each passed their one selected test. The allowed-to-fail lint step still
  reports only two overlong comments (`src/libc/dlfcn.rs:61` and
  `src/frameworks/foundation/ns_dictionary.rs:563`). The development workflow
  skips the guest executable integration and native macOS guest sanity steps.
  Android cross-compilation is not claimed because its allowed lint failure
  occurs before a reliable cross-compilation result.
- [IPA build 41294836557](https://github.com/bookieart1/iDared32bit/actions/runs/41294836557)
  completed arm64 iOS compilation, development-identity checks, unsigned
  packaging and artifact upload. Artifact `13078298211` is named
  `iDared32bit-dev-unsigned-178cc5197ebc80415538d12aaa6458d3a3e47abc`.
  Its downloaded ZIP SHA-256
  `910b341dbc0dcb4616939aa82936474c869b3e443aa67057e32343f3cf348634`
  matches GitHub's published digest.
- Local `outputs/iDared-Dev-1.81.ipa` is 19,924,279 bytes with SHA-256
  `30880c66ca5c286f8e872efcf8aa969f6e6bbc4c26805dc85e5cbc8d4e0a82f8`.
  All 36 archive entries passed CRC validation. The package contains one
  arm64 executable, has no Mach-O code-signature command, provisioning profile
  or `_CodeSignature` directory, and retains bundle identity
  `com.bookieart1.idared32bit.dev` / `iDared Dev`. Embedded markers confirm
  source `178cc51`, workflow `41294836557`, the universal flight recorder, all
  three fifth-stage contracts, picker page persistence, physical-orientation
  updates and viewport touch clamping.

Verification record: `outputs/iDared-Dev-1.81.verification.json`. Package and
hosted-test verification do not establish physical-device compatibility; every
prediction above remains pending the focused iPhone run.

## Focused device procedure

Without deleting saves:

1. Blot: play briefly and use emulator Home.
2. Dark Nebula 2: select a level.
3. Dirt: move and slowly rotate the phone through the four upright
   orientations; confirm gravity follows. While holding movement toward one
   side, slide the finger from the game into that side's black bar and confirm
   the character does not reverse. Then pause/resume, use Home, reopen and
   confirm the position.
4. Glowfish: launch and try to enter gameplay.
5. Glowfish HD: press Play, enter a name if asked, and continue.
6. Hamlet: launch and navigate briefly; note whether the opening orientation
   corrects itself.
7. Parachute Ninja: launch once and allow the first error.
8. Spoing: play briefly, use Home and reopen.
9. From a later picker page, swipe one page slowly enough to watch the icons
   follow the finger. Launch any game there, then exit or let a known failure
   occur and confirm the picker returns to that same page. Force-close and
   reopen iDared once and confirm the page is still retained.

Stop each failing title after its first reproducible crash and preserve the
complete combined log.
