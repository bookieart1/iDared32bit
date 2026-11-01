# Dev 1.79: third-stage runtime, archive, file and UIKit contracts

## Evidence and scope

The physical-device input is the unmodified Dev 1.78 log now preserved outside
the repository as `outputs/device-logs/iDared-Dev-1.78-device-results.txt`:
544,116 bytes, SHA-256
`c40ca031bcdad068b7c5c1db56662d5fdf158b929f6fa8de715f2c682e916ac3`.
It embeds source `1af64fa` and workflow `41287654525`. The universal bounded
flight recorder is active for every relevant launch.

This iteration addresses five independent, generally applicable contracts. It
does not change the recorder, add application-identifier branches, relax memory
safety, swallow failures or delete/migrate saves.

| App | Dev 1.78 decisive evidence | Dev 1.79 implementation | Boundary |
|---|---|---|---|
| Dark Nebula 2 | Its menu rendered and ran for 6.816 seconds, 353 main-loop iterations and 179 EAGL returns before `_ivar_getTypeEncoding` was missing. The preceding `object_getInstanceVariable` returned a real ivar handle. | `ivar_getTypeEncoding` now returns the stable compiler type string from that ivar metadata and returns null for a null handle. | Verified runtime contract in focused guest coverage; later gameplay remains unknown. |
| Dirt | Smooth gameplay, pause/resume and the user-observed stage-transition attempt reached `NSKeyedArchiver encodeBool:forKey:` on thread 6 after 27.984 seconds, 1,652 loops and 1,651 EAGL returns. Recorder arguments show boolean `1`. | Keyed archives store boolean scalars as plist booleans; existing unarchive support reads them. | Archive round-trip covered for false and true; what follows the save remains unknown. |
| Glowfish / Glowfish HD | Both passed their previous VM/pthread blockers, then failed before a loop or frame at `_fchmod`, immediately after obtaining and locking a file descriptor. | `fchmod` validates descriptors, applies Unix permission bits to writable host-backed files, reports read-only/socket/I/O errors, and preserves the existing single-user guest model for directory descriptors. | Full on iOS/macOS host-backed files; directory mode persistence and non-Unix permission bits remain explicitly partial. |
| Hamlet | A valid `UINavigationController` received `setNavigationBarHidden:YES animated:NO` 113 ms after launch. | The navigation-bar-hidden property now has observable state and the animated setter applies the same final state synchronously. | Property/final state implemented; navigation-bar rendering and animation are still unsupported. |
| Parachute Ninja | `_UIApplicationLaunchOptionsURLKey` remained an unresolved non-lazy symbol. Launch then dereferenced the null global in `NGAppPatch application:didFinishLaunchingWithOptions:` and faulted before the first loop/frame. | UIKit now exports the historical launch-options URL key as an NSString constant. | Constant identity/value contract implemented; handling any actual launch URL remains the application's responsibility. |

Blot ran smoothly through the requested play/death path and returned through
emulator Home after 35.533 seconds (1,275 loops, 1,233 EAGL returns). Spoing ran
smoothly and returned through Home after 41.418 seconds (1,132 loops, 522 EAGL
returns). They remain the focused regression controls.

## Validation plan and predictions

Focused guest tests cover null and valid ivar type lookup, keyed boolean
round-trips, writable-file `fchmod` plus invalid-descriptor errno, both
navigation-bar-hidden setters/getter, and the UIKit constant. Hosted library
tests validate compilation and the emulator's unit suite; the repository's
development-branch workflow still skips the guest executable, so guest test
source compilation and physical-device behavior must be reported separately.

Predictions for Dev 1.79:

- Dark Nebula 2 passes the ivar-type query and advances beyond its current
  gameplay-start crash.
- Dirt passes the boolean archive operation during the map/save transition.
- Glowfish and Glowfish HD pass `fchmod`; visible launch is not yet predicted.
- Hamlet passes the hidden-navigation-bar call; visible launch is not yet
  predicted.
- Parachute Ninja no longer dereferences a null launch-options-key global.
- Blot and Spoing retain their stated working behavior.

A removed blocker, a visible stage advance and functional compatibility remain
separate results. The next device log must be used to update these contracts and
predictions rather than treating successful compilation as compatibility.
