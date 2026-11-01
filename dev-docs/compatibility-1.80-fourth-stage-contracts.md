# Dev 1.80: fourth-stage compatibility contracts

## Evidence and scope

The unmodified Dev 1.79 device log is preserved outside the repository as
`outputs/device-logs/iDared-Dev-1.79-device-results.txt`: 781,817 bytes,
SHA-256
`9c8c2c78d2cef030b7a14b63ec082ad4a1246499f4ea671f1198064cdfa1c5cc`.
It embeds implementation source `4de150e7` and workflow `41289274667`. The
universal bounded flight recorder was active for every relevant launch.

This iteration addresses four independent, generally applicable contracts. It
does not change the recorder, add application-identifier branches, relax
memory safety, manufacture network success, or delete/migrate saves.

| App | Dev 1.79 decisive evidence | Dev 1.80 implementation | Boundary |
|---|---|---|---|
| Dark Nebula 2 | The menu rendered and ran for 6.842 seconds, 350 main-loop iterations and 178 EAGL returns. Gameplay loading then called missing `_ivar_getOffset` with the real ivar metadata reached by the prior runtime work. | `ivar_getOffset` reads the stable compiler-provided offset and returns zero for a null handle. | Runtime query and null behavior are covered; the next gameplay-loading stage is unknown. |
| Glowfish / Glowfish HD | Both passed extensive resource loading and reached a valid `CMMotionManager`, then failed at `startAccelerometerUpdates` before the first loop/frame. | A per-manager polling accelerometer model implements start, stop, active state, interval, latest data, timestamp and the `CMAcceleration` struct, using the emulator's existing real/simulated sensor source. | Polling is implemented. Operation-queue/handler delivery, gyro and full device-motion fusion remain unsupported. |
| Hamlet | A valid `UINavigationController` received missing `navigationItem` after 113 ms, before the first loop/frame. | Every `UIViewController` lazily owns one stable `UINavigationItem`. Controller title copying/propagation and the item's historical title, prompt, title view, bar-button-item and hidden-back-button state are retained with synchronous animated setters. | State/ownership are implemented; navigation-bar rendering and animations are not claimed. |
| Parachute Ninja | Launch reached a genuine offline `NSError`, but `_NSErrorFailingURLStringKey` and `_NSURLErrorDomain` were unresolved; guest error handling then dereferenced the null domain global and faulted after 104 ms. | Foundation exports the historical URL-error domain, failing-URL-string key and newer alias. Synchronous URL failures attach the failed URL string while retaining the genuine offline/file error. | Error metadata is implemented; networking remains offline and no service success is fabricated. |

Dirt is now a **working checkpoint in the observed scope**: the user moved
through several areas, paused/resumed, exited, reopened, and recovered the
saved position. Its two launches returned cleanly through emulator Home after
113.893 seconds (6,781 loops, 6,779 EAGL returns) and 5.297 seconds (293 loops,
292 EAGL returns). This is not a claim that every level or mechanic is defect
free.

Blot completed the requested smooth play/death path and clean Home return after
60.495 seconds (3,521 loops, 2,649 EAGL returns). Spoing remained smooth and
returned through Home after 46.129 seconds (1,392 loops, 642 EAGL returns).
They remain focused regression controls.

## Validation plan and predictions

Focused guest coverage now exercises valid/null ivar offsets, Core Motion
polling start/data/stop plus the three-double return ABI, navigation-item
identity/title propagation/retained state, and URL-error constants and failed
URL metadata. The development-branch hosted workflow runs library tests but
still skips the guest executable, so successful hosted compilation and
physical-device behavior must be reported separately.

Predictions for Dev 1.80:

- Dark Nebula 2 passes `_ivar_getOffset` and advances beyond its current
  gameplay-loading crash.
- Glowfish and Glowfish HD pass accelerometer startup and can poll the same
  sensor values used by `UIAccelerometer`; visible gameplay is not yet proven.
- Hamlet passes `navigationItem` and can observe coherent navigation state;
  visible launch is not yet proven.
- Parachute Ninja processes the real offline error without a null constant;
  the unavailable service may still be an application-level blocker.
- Dirt, Blot and Spoing retain their stated behavior without save reset.

A removed blocker, a visible stage advance and functional compatibility remain
separate results. The next complete device log must update these predictions.

## Focused device procedure

Test alphabetically without deleting saves:

1. Blot: play until death, then use emulator Home.
2. Dark Nebula 2: enter the menu and select a level.
3. Dirt: move through several areas, pause/resume, exit, reopen, and confirm
   that the position is retained.
4. Glowfish: launch and attempt to reach/play gameplay.
5. Glowfish HD: repeat the same short launch/gameplay test.
6. Hamlet: launch and navigate the first interface if it appears.
7. Parachute Ninja: launch; if a menu appears, try starting gameplay.
8. Spoing: play briefly, exit, and reopen.

Stop each failing title after its first reproducible crash and preserve the
complete combined log.
