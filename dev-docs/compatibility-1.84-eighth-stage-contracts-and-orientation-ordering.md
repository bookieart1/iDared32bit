# iDared Dev 1.84: eighth-stage contracts and orientation ordering

Date: November 1, 2026

## Evidence and implementation boundary

The complete Dev 1.83 device log is archived unchanged as
`outputs/device-logs/iDared-Dev-1.83-device-results.txt`: 874,547 bytes,
SHA-256
`4f0c7442591e142bde9777e92f5ee01fab552b231ea30e720e0c4cfff6a9c147`.
It embeds source `7d6a55ebba55488762bc890452d0768b0740ad0b`, workflow
`41300999509`, and the unchanged universal bounded recorder.

Dev 1.84 contains two shared Foundation contracts and one host-orientation
ordering correction. It does not add application identifiers, tolerate
invalid objects, suppress aborts, invent resources, fabricate network success,
alter saves, or change screenshot suspension or the recorder.

## Dev 1.83 device result

| App / surface | Observed result | First decisive evidence and classification |
|---|---|---|
| Blot | Smooth play and clean emulator-Home return after 70.900 seconds, 3,814 loops and 3,204 EAGL returns. | **Regression pass in the stated scope.** |
| Dark Nebula 2 | Menu, gameplay, a complete level and name entry worked. After the user declined automatic score submission, the game reconstructed a saved leaderboard date and sent missing `dateFromString:` to a valid `NSDateFormatter` after 196.587 seconds / 11,660 loops / 5,717 EAGL returns. | **Major compatibility advance; partial post-level blocker.** This is local date parsing, not evidence of a network crash. |
| Dirt | Continued to work and returned cleanly through Home after 58.001 seconds / 3,945 loops / 3,441 EAGL returns. | **Regression pass in the expanded tested scope.** |
| Glowfish | Startup advanced to valid `NSProcessInfo` receiving missing `globallyUniqueString` after 1.308 seconds, before a main loop and after one EAGL return. | **Startup-blocked at a shared Foundation contract.** |
| Glowfish HD | Name/menu/intro/level selection remain reachable. Starting the level again failed to open bundled `GameData/Levels/Level_01.gfl`, then the guest deliberately called `abort` after 29.425 seconds / 1,352 loops / 1,261 returns. | **Partial; genuine resource/package failure remains strict.** The native keyboard remains portrait. |
| Hamlet | Ran and returned cleanly after 7.737 seconds / 1,100 loops / 408 returns. | **Working checkpoint with unchanged initial orientation defect.** |
| Parachute Ninja | The guest again sent `domain` to a supplied `UIApplication` after 144 ms. | **Blocked at unchanged invalid callback-argument origin.** No error methods are added to UIApplication. |
| Spoing | The log reaches a screenshot interruption, records resign-active, pauses the guest, records did-become-active, resumes the same guest and later records 592 loops / 273 EAGL returns. It ends while that resumed session is still running, with no panic or picker return. | **Immediate screenshot suspend/resume succeeded.** A later host close is not established by this truncated log, so the existing lifecycle code is preserved. |

The old `Host orientation bridge is unavailable` warning is absent, proving
that Dev 1.83 connected the callback. The visual defects nevertheless remain.
Source order explains the next testable hypothesis: the callback ran
synchronously immediately before SDL constructed its game window, so the
scene update could observe the departing picker rather than SDL's root view
controller.

## Dev 1.84 changes

### Foundation date parsing

`NSDateFormatter dateFromString:` now parses the bounded en_US subset of the
Unicode date patterns already supported for formatting: numeric and named
dates, 12/24-hour time, fractional seconds, quoted literals, weekday text and
GMT/numeric time-zone forms. It honors an explicit parsed zone or the
formatter's configured/default zone, constructs a real `NSDate`, validates
calendar ranges and returns `nil` for invalid or unsupported input. It does
not guess through malformed data.

This predicts that Dark Nebula 2 can finish its local post-level bookkeeping;
the next later blocker, if any, remains unknown.

### Globally unique process strings

`NSProcessInfo globallyUniqueString` now returns a fresh autoreleased uppercase
RFC 4122 v4 string for every request. It provides uniqueness without exposing
a stable host/device identifier. This predicts that Glowfish passes the exact
startup blocker seen in Dev 1.83; it does not predict complete gameplay.

### Host scene orientation ordering

The scene geometry request is now always queued for the next main-run-loop
turn. SDL's orientation hint is still installed synchronously, but the native
scene request consequently sees SDL's newly installed root controller after
the portrait picker is dismissed. Bounded one-shot logs record requested,
current and resulting host orientation plus the root-controller type. This is
an ordering correction and discriminator, not a title-specific rotation rule.

The next device log can therefore distinguish an accepted scene rotation from
a remaining controller/mask problem. Glowfish HD and Hamlet still require
visual confirmation; a successful build cannot establish their layout.

## Validation plan

- Rust unit coverage exercises valid numeric/named/12-hour/time-zone/fractional
  date parsing and invalid calendar/input rejection.
- The existing native Foundation fixture now checks date parse/format
  round-tripping, invalid-date rejection and per-call process-string
  uniqueness.
- Hosted library tests, lint diagnostics, iOS compilation and package checks
  remain separate. The development workflow skips the full guest executable
  integration path.

## Focused device test

Keep existing saves and test alphabetically:

1. **Blot:** play briefly and return with emulator Home.
2. **Dark Nebula 2:** finish one level, enter a name, choose **No** at score
   submission, and continue until either the next screen or first failure.
3. **Dirt:** move and rotate briefly, then return with emulator Home.
4. **Glowfish:** launch and continue until gameplay or the first failure.
5. **Glowfish HD:** reach name/level selection, note keyboard orientation and
   try starting a level once.
6. **Hamlet:** note whether the very first screen is correctly landscape, then
   return with emulator Home.
7. **Parachute Ninja:** launch once and allow the first error.
8. **Spoing:** play briefly and return with emulator Home. No separate
   screenshot test is required in this iteration.

One complete combined log is sufficient. No save deletion is requested.
