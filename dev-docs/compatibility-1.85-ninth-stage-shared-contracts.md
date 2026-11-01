# Dev 1.85: ninth-stage shared contracts and keyboard-regression recovery

## Evidence and scope

The three Dev 1.84 device logs embed source
`e5045fb063ce2d0c0b8f870ac29feae79c43510f`, workflow `41303668226`, and
the universal bounded recorder. They are archived as
`outputs/device-logs/iDared-Dev-1.84-device-results-part-{1,2,3}.txt`.

| Part | Bytes | SHA-256 | Tested scope |
|---|---:|---|---|
| 1 | 73,162 | `00e3deb5530b09ef2c00ad96a29355f50ad69200c8fd17c777cc447b4067c3c4` | Blot; Dark Nebula 2 through a complete level and name-entry request |
| 2 | 365,440 | `97b7532e4957ae39f0a694bffa19dd8b5a280e5a0fc2a6ce740516e908a2729f` | Glowfish; Glowfish HD through name-entry request |
| 3 | 222,527 | `bc070ae29c4efec50c65004e687e7cf511a6d5f51f0ace02eb5c4b5c11552e3a` | Hamlet; Parachute Ninja; Spoing |

Dev 1.85 deliberately combines four small, independently testable framework
contracts with recovery from one observed host regression. It does not alter
the diagnostic recorder, network availability, resource lookup, memory-fault
policy, or app-specific behavior.

## Findings

- **Blot** remains a regression pass. It played normally and returned through
  emulator Home.
- **Dark Nebula 2** again completed a level. Requesting its name-entry UI then
  terminated the whole host; the log ends immediately after configuring the
  text field, without a guest panic or Home event.
- **Glowfish** advanced beyond the Dev 1.84 process-identity blocker and now
  fails because valid `NSDate` lacks `encodeWithCoder:` during keyed archival.
- **Glowfish HD** reaches its usual menu, but requesting the same native
  name-entry UI also terminates the host without a guest panic. The matching
  transition in two unrelated games, and its first appearance after the Dev
  1.84 deferred scene-geometry change, make that change the strongest shared
  regression candidate. This is not proof of a particular UIKit exception
  because iOS did not append a host crash report to the guest log.
- **Hamlet** remains playable after the same transient rotated/cropped initial
  presentation. Dev 1.84 did not improve it.
- **Parachute Ninja** still supplies a real `UIApplication` to code that sends
  `domain`. The recorder establishes the invalid receiver and the preceding
  offline paths, but not a safe replacement origin. Adding error selectors to
  UIApplication would conceal the invalid value and is rejected.
- **Spoing** plays normally and exits cleanly.

The two recent survey samples also establish three exact shared contracts:

- Toy Bot and Toy Bot 2 both stop at `UISlider.setContinuous:`. Crazy Golf is
  already working in the tested gameplay path and is an extended-settings
  regression for the same contract.
- Fragger reaches its menu and then stops at `UIFont.fontName`; Cube has the
  same exact missing selector.
- Rolando and junny both stop at
  `NSRunLoop.cancelPerformSelectorsWithTarget:`.

## Implementation boundary

- `NSDate` now encodes its reference-date interval under the historical
  `NS.time` keyed-archive key; `NSKeyedArchiver` gains the required double
  encoding primitive. This matches the existing decoder instead of inventing
  a game-specific archive.
- `UISlider` now stores the historical continuous-event flag, defaulting to
  true, and supports `isContinuous` / `setContinuous:`. Rendering and touch
  behavior remain outside this narrow contract.
- `UIFont` now exposes stable public font identity and preserves it across
  `fontWithSize:`. System fonts use historical Helvetica PostScript names;
  recognized app-font requests retain the requested registered name.
- `NSRunLoop` now cancels all queued perform requests whose target is identical
  to the supplied target, releasing the queue's retains and waking any waiter.
- The unsuccessful Dev 1.84 deferred host-geometry ordering is reverted to the
  Dev 1.83 synchronous-main-thread behavior. This predicts restored keyboard
  entry, but does not claim to correct Hamlet's or Glowfish HD's orientation.
- Parachute Ninja remains strict and unchanged pending evidence for the origin
  of its invalid callback value.

Focused unit tests cover date archive round-trip, slider default/mutation,
font identity across resize, and target-specific run-loop cancellation.

## Verified delivery

- Final implementation/source commit:
  `84c7ca2b15962b7d73f8de6d33130e018ef2d6b3`. The principal implementation is
  in `ed4c0c32`; the two later commits correct only the new test imports and
  synthetic guest-stack fixtures discovered by the first hosted runs.
- [Hosted tests 41307145921](https://github.com/bookieart1/iDared32bit/actions/runs/41307145921):
  **142/142 library tests passed on macOS and 142/142 on Windows**. The native
  Foundation reference and the two selected recorder reproducers passed.
  Development-branch full guest-executable tests were skipped. The separately
  allowed-to-fail macOS, Windows and Android lint steps each emitted an
  exit-code-1 annotation, so the green workflow is not a clean-lint claim.
- [IPA build 41307145898](https://github.com/bookieart1/iDared32bit/actions/runs/41307145898):
  arm64 compilation, separate development identity verification, unsigned
  packaging and artifact upload passed. Artifact ID `13083642786`; downloaded
  ZIP SHA-256
  `5ce6ecef17d15faae07760b6336c6dda413be0c738b84ae3b51e572ffce2ebd8`
  matches GitHub's published digest.
- Local `outputs/iDared-Dev-1.85.ipa`, 19,967,511 bytes, SHA-256
  `76fd30f74e0028e37d7676df7df7dff9e084407b3bc0d64f042a1ce5ec6c0251`,
  passed archive extraction, arm64 Mach-O, unsigned/no-provisioning, separate
  `com.bookieart1.idared32bit.dev` identity, embedded source/workflow,
  universal-recorder and new-contract marker checks.

Verification: `outputs/iDared-Dev-1.85.verification.json`. Physical-device
compatibility remains pending and separate from these build and host-test
checkpoints.

## Device predictions

- Dark Nebula 2 and Glowfish HD should once again show usable name-entry UI
  rather than terminating iDared. The older orientation defect may remain.
- Glowfish should pass its keyed-date archival call and expose a later blocker
  or begin presenting normally.
- Toy Bot / Toy Bot 2, Fragger / Cube, and Rolando / junny should all pass their
  respective exact missing selectors. A passed selector is not by itself a
  gameplay claim.
- Blot, Dirt, Crazy Golf and Spoing are the focused regressions. No save reset
  is requested.

