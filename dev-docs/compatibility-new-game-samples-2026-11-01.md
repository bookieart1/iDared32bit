# New-game compatibility samples — November 1, 2026

This note reconciles two physical-device samples that were not part of the
Dev 1.71 full-catalog survey. It records observed behavior separately from the
first explicit emulator blocker. Neither sample authorizes or contains a
compatibility patch.

## Evidence

- Dev 1.82 sample: source `99580ce5a9230eb3fb7c51588412e6cc93851d51`,
  workflow `41298463255`, 1,010,202 bytes, SHA-256
  `9ac324298b6f669cd80ea0af679497a915dc9bf6898b5126d8a39606b28ddeac`.
- Dev 1.83 sample: source `7d6a55ebba55488762bc890452d0768b0740ad0b`,
  workflow `41300999509`, 748,464 bytes, SHA-256
  `2828f76444b586a92c88338763f369f85aa779aba022000057e6f1eccaa36dc5`.
- The bounded universal recorder was enabled for every decrypted title that
  entered CPU emulation. The encrypted 8bitninja 1.4 executable stopped before
  emulation and therefore has no recorder history.
- Archived logs and the machine-readable inventory are under `outputs/` in the
  project workspace. The two pre-existing holistic surveys remain unchanged.

## Dev 1.82 sample

| App | Device observation and terminal result | First explicit blocker / classification |
|---|---|---|
| Bumpy Road 2.0 | User observed working play before an apparent exit. The log proves an on-screen Home event, not a crash. | Working checkpoint in the tested scope; longer play and save/relaunch were not established. |
| Max & the Magic Marker 1.0.11 | Reached 16 main loops and 10 EAGL returns, then panicked after 2.161 s. | Missing libc `_nl_langinfo`; locale-information contract. |
| Oscura 5.1 | User reported smooth gameplay, menus and pause; two launches ended through Home. | Working checkpoint in the tested scope. |
| RastaMonkey 1.6.5 | User reported smooth gameplay and menus; clean Home return. | Working checkpoint in the tested scope. |
| Snail Mail 1.9.0 | User reported smooth gameplay and menus; clean Home return. | Working checkpoint in the tested scope. |
| SpaceMnkey 1.4 | Reached 78 main loops and 24 EAGL returns, then panicked after 1.908 s. | Missing libc `_mbsrtowcs`; exact blocker shared with Zombie Attack! 2nd Wave and part of the wider locale/wide-character family. |
| Slingshot 1.1 | Failed before a main loop with a guest null dereference after an Objective-C `objectAtIndex:` path. | Invalid-object/null-origin family; the recorder does not yet establish which earlier return supplied the null. |
| SUPERFALL 1.0.3 | Reached four main loops and one EAGL return, then panicked. | Missing `NSNumberFormatter`; exact class blocker shared with Silverfish. |
| Sway 1.2.2 | User reported smooth gameplay and menus; two launches ended through Home. | Working checkpoint in the tested scope. |

The log contains nine distinct titles even though the contemporaneous verbal
summary described eight. Slingshot is retained because it is an actual launch
in the supplied evidence.

## Dev 1.83 sample

| App | Progress and terminal result | Correlation and current interpretation |
|---|---|---|
| 1000:Find'Em 1.2.0 | Two reproducible startup panics, both before a main loop or EAGL return. | Detached-view/window coordinate conversion: `AppView.window` is nil and conversion asserts. Exact family shared with Glyder, Glyder 2 and JumpO'Clock; all four are Glu titles. |
| 8bitninja 1.4 | Rejected before CPU emulation because the executable is encrypted. | Packaging/input limitation, not an emulator compatibility result. |
| 8bitninja 1.2 | Startup panic before a main loop or EAGL return. | `CFDictionaryCreateMutable` rejects positive capacity with null callbacks. Exact family shared with DoDoFly. Capacity and callback ownership semantics must be implemented together. |
| Fragger 1.4.1 | User saw a functional menu; level loading later failed after 5.404 s, 293 main loops and 293 EAGL returns. | Valid `UIFont` lacks `fontName`. Exact contract shared with Cube. This is not an instant-startup failure. |
| JellyCar 3 1.2 | Startup panic before a main loop. | Missing `UINavigationBar` class. Exact family shared with Decoloro; this is a larger UIKit component rather than a one-selector stub. |
| Peggle 3796 | One main loop, no EAGL return, then panic. | Missing `NSBundle pathForResource:ofType:inDirectory:forLocalization:`. New explicit contract in the recorded survey evidence; localization and search-order semantics matter. |
| Ragdoll 3 1.2.1 | Startup panic before a main loop or EAGL return. | Unknown `sysctlbyname` key `hw.activecpu`. New explicit key; a consistent virtual CPU-count contract is likely bounded, but historical behavior still needs verification. |
| Rolando 1.9 | Startup panic before a main loop or EAGL return. | Missing `NSRunLoop cancelPerformSelectorsWithTarget:`. Exact contract shared with junny; cancellation ordering is semantically important. |
| Topple 1.9.2 | One main loop and one EAGL return, then strict dictionary panic on nil value for `If-None-Match`. | The failure follows a synchronous ngmoco service request returning nil. The dictionary correctly rejects nil; the unresolved problem is the preceding URL/cache error-path value and control flow. |
| Topple 2 1.3 | One main loop and one EAGL return, then `maybe_initialize_class` could not resolve its receiver as a tracked Objective-C host object. | Dispatch/class-metadata or invalid-object-origin family. Current evidence does not prove the same cause as Topple, despite the shared publisher/service path. |
| Toy Bot 1.4 | Startup panic before a main loop or EAGL return. | Valid `UISlider` lacks `setContinuous:`. Exact selector shared with Crazy Golf. Existing slider rendering/control behavior remains incomplete, so this setter alone would not prove full compatibility. |
| Toy Bot 2 1.4 | Same startup panic as Toy Bot. | Same `UISlider setContinuous:` family; a single general contract should cover both titles and Crazy Golf. |

## Engineering implications

The strongest shared-contract candidates are `UIFont.fontName`,
`UISlider.setContinuous:`, `_mbsrtowcs`, `NSRunLoop` cancellation, the
detached-view conversion contract, and Core Foundation dictionary
capacity/callback semantics. `NSBundle` localized resource lookup,
`hw.activecpu`, and `_nl_langinfo` are new but general explicit blockers.

The two Topple failures and Slingshot should not be patched by weakening strict
dictionary or memory checks. They need provenance-focused diagnosis of the
invalid/nil origin. `UINavigationBar` is general but materially larger and more
regression-prone than a missing metadata accessor. Encryption remains outside
compatibility implementation.

No title is promoted beyond the user's observed scope. Clean Home returns show
that the emulator did not panic; they do not independently prove every level,
audio path, save path or relaunch path.
