# iDared Dev 1.83: seventh-stage contracts and suspend recovery

Date: November 1, 2026

## Evidence and implementation boundary

The complete Dev 1.82 device log is archived unchanged as
`outputs/device-logs/iDared-Dev-1.82-device-results.txt`: 1,093,375 bytes,
SHA-256
`5cdfb404ed870e53430d0594d523c11a81bb898e5ed6eeea733a0a428cc4777c`.
It embeds source `99580ce5a9230eb3fb7c51588412e6cc93851d51`, workflow
`41298463255`, and the universal bounded recorder.

Dev 1.83 contains shared Foundation and host-lifecycle corrections only. It
does not add game identifiers, tolerate invalid objects, suppress `abort`,
invent missing resources, alter saves, or change the recorder.

## Dev 1.82 device result

| App / surface | Observed result | First decisive evidence and classification |
|---|---|---|
| Blot | Smooth play and clean emulator-Home return after 52.418 seconds, 1,839 loops and 1,792 EAGL returns. | **Regression pass in the stated scope.** |
| Dark Nebula 2 | Menu and level-loading work continued for 18.271 seconds / 1,000 loops / 505 EAGL returns, then a mutable array received missing `removeObjectsAtIndexes:`. | **Partial; later shared Foundation blocker.** |
| Dirt | Gravity, traversal and input remained functional; clean Home return after 63.999 seconds / 3,984 loops / 3,801 EAGL returns. | **Working checkpoint in the expanded tested scope.** |
| Glowfish | Resource and offline-callback initialization advanced to a mutable dictionary receiving missing `keysSortedByValueUsingSelector:` after 1.288 seconds. | **Startup-blocked at a standard Foundation sorting contract.** |
| Glowfish HD | Menu/name/intro/level-selection paths remain reachable. Level start again failed to open bundled `GameData/Levels/Level_01.gfl`, logged the failure, then deliberately called `abort` after 27.970 seconds / 1,315 loops / 1,183 EAGL returns. The native keyboard remained portrait. | **Partial.** The resource failure is genuine and remains strict; keyboard orientation is a separate host integration failure. |
| Hamlet | Runs and exits cleanly after 12.481 seconds / 1,445 loops / 705 EAGL returns, but its first presentation remains rotated/cropped. | **Working checkpoint with verified initial orientation defect.** |
| Parachute Ninja | Guest code again received `UIApplication` where it expected an error and sent it `domain`, panicking after 140 ms. | **Startup-blocked at an unchanged invalid callback argument.** No error methods will be added to UIApplication. |
| Spoing | Two normal launches worked and returned cleanly (7.773 seconds / 175 loops / 84 returns; 9.860 seconds / 302 loops / 143 returns). | **Regression pass twice in the stated scope.** |
| Screenshot review | A third Spoing launch received SDL's host-background event after 6.717 seconds. Dev 1.82 deliberately ended the guest and returned to the picker; there was no emulator panic. | **Host preserved, guest not preserved.** This confirms the previous fallback policy rather than a guest compatibility crash. |

Every relevant launch logged `Host orientation bridge is unavailable; leaving
scene orientation unchanged`. This is decisive evidence that Dev 1.82's
runtime symbol lookup never connected the Rust emulator to the Swift scene;
the unchanged Glowfish HD keyboard and Hamlet presentation therefore do not
test the intended scene request itself.

## Dev 1.83 changes

### Mutable-array indexed removal

`NSMutableArray removeObjectsAtIndexes:` now validates the complete index set
before mutation and removes original indexes in descending order through the
array's primitive method. This preserves index meaning and subclass behavior;
it does not special-case Dark Nebula 2. Focused coverage verifies ordering and
object ownership.

### Dictionary key sorting by values

`NSDictionary keysSortedByValueUsingSelector:` now collects keys through the
dictionary primitives, compares their corresponding values with the supplied
Objective-C selector, and returns an autoreleased array that owns its keys.
Focused coverage sorts numeric values and verifies the returned keys. This is
the exact general contract reached by Glowfish.

### Reliable host-scene orientation registration

The final iOS executable now registers the Swift scene-orientation function
with Rust directly before SDL starts, using the same callback architecture as
the native picker and text alert. The unreliable cross-image `dlsym` lookup
and its forced-link flags are removed. The guest's existing orientation choice
remains authoritative; no title-specific orientation is imposed.

The device discriminator is simple: the old `Host orientation bridge is
unavailable` line must disappear. The keyboard and Hamlet still require visual
device validation because successful registration does not by itself prove
that iOS accepted a particular scene geometry request.

### Transient host interruption pause/resume

The host-background event now sends the normal
`applicationWillResignActive:` callback and notification, pauses all guest
execution, and keeps SDL's host event pump alive. On foregrounding it sends
`applicationDidBecomeActive:` and resumes the same guest. Real host
termination still follows the strict termination/save path.

This is deliberately bounded suspend recovery, not background multitasking:
the guest does no work while inactive, and UIKit background-task execution is
not claimed. A guest exception in either lifecycle callback remains visible.

## Focused device test

Keep existing saves and test alphabetically:

1. **Blot:** play briefly, then return with emulator Home.
2. **Dark Nebula 2:** select a level and stop at the first failure.
3. **Dirt:** move, rotate for gravity, then return with emulator Home.
4. **Glowfish:** launch and try to reach gameplay.
5. **Glowfish HD:** reach name entry/level selection and start a level; note
   the keyboard orientation.
6. **Hamlet:** note whether the very first screen is correctly landscape.
7. **Parachute Ninja:** launch once and allow the first error.
8. **Spoing:** play briefly and return with emulator Home.
9. In Spoing or another stable game, take and save a screenshot. Return to
   iDared and confirm whether the **same running game** resumes.

The universal recorder remains enabled. One combined log is sufficient.

## Validation status

Final implementation commit: `7d6a55ebba55488762bc890452d0768b0740ad0b`.

- [Hosted tests 41300999515](https://github.com/bookieart1/iDared32bit/actions/runs/41300999515):
  137/137 development-library tests passed on macOS x86_64 and 137/137 on
  Windows. The native Foundation reference, both bounded diagnostic
  reproducers and the Android compile/lint job passed. The branch-gated full
  guest-executable steps were skipped, not counted as passes.
- [IPA build 41300999509](https://github.com/bookieart1/iDared32bit/actions/runs/41300999509):
  arm64 iOS compilation, separate development identity checks, unsigned
  packaging and artifact upload passed. Artifact ID `13081776006`,
  `iDared32bit-dev-unsigned-7d6a55ebba55488762bc890452d0768b0740ad0b`;
  its 19,912,927-byte ZIP has SHA-256
  `ea4aaa25c738cec94481f2b06c7668ac3dc0998097dd0011d5d496abbcb6f933`,
  matching GitHub's published digest.
- Local `outputs/iDared-Dev-1.83.ipa` is 19,959,152 bytes with SHA-256
  `38f0f199075e5b1157e6047551d1e700f49297c7271c657d20988b677fff0c75`.
  Independent extraction verifies the embedded source and workflow, arm64
  executable, `com.bookieart1.idared32bit.dev` identity, universal recorder,
  no code signature and no provisioning profile.

Verification is preserved in `outputs/iDared-Dev-1.83.verification.json`.
Physical-device compatibility remains pending and must not be inferred from
these build and host-test results.
