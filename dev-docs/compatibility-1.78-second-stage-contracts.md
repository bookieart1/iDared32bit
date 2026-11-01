# Dev 1.78: second-stage contracts from the first priority batch

Date: 2026-11-01

## Evidence and scope

The physical-device log is from Dev 1.77 source `7640156` and workflow
`41284825031`. Its unmodified copy is preserved at
`outputs/device-logs/iDared-Dev-1.77-device-results.txt` outside this checkout
(530,642 bytes; SHA-256
`b608c4f5d259f50ace6cbb26f1b360a4d5aa4388637a20f69ed6802994fc0dde`).
The universal bounded flight recorder was active for every relevant launch.

Dev 1.77 removed all five blockers that it targeted, but only Dark Nebula 2
reached a visible menu. The next first decisive blockers are independent API
contracts rather than one common failure family:

| App | Verified Dev 1.77 result | Dev 1.78 contract |
|---|---|---|
| Dark Nebula 2 | Smooth menu and level selection; after 13.895 s, 777 main loops and 391 EAGL returns, level startup called missing `_object_getInstanceVariable`. | Look up binary ivar metadata through the object's class hierarchy, return the stable `Ivar`, and copy the pointer-sized value to a nullable output pointer without retaining it. |
| Dirt | The new NSData URL/options reader returned 1,474 bytes. XML initialization converted NSString encoding 4 to CF UTF-8, then called missing `_CFStringConvertEncodingToIANACharSetName`. | Return Core Foundation's canonical IANA names for every encoding already supported by this module; unknown encodings return NULL. |
| Glowfish / Glowfish HD | The VM-tagged anonymous mapping blocker was removed. Both entered bundled runtime/thread initialization and stopped at missing `_pthread_setcancelstate`. | Store cancellation state and type per emulated pthread, report prior values, validate inputs with `EINVAL`, and preserve POSIX defaults. Cancellation request/delivery remains unsupported and is not fabricated. |
| Hamlet | UIImage inherited `description` and NSLog now complete. The next call sends `drawAsPatternInRect:` to that valid image. | Tile the image at native size from the rectangle origin, clip partial edge tiles, preserve graphics state, and safely ignore a nil context or invalid/empty geometry. |
| Parachute Ninja | `_sel_getUid`, both method lookups and both implementation reads complete. It then replaces a UIApplication implementation with a guest implementation through missing `_method_setImplementation`. | Canonical runtime `Method` handles now update the owning method table and return the previous IMP. Host methods expose a direct callable trampoline rather than `_objc_msgSend`, so a saved original IMP does not redispatch into the replacement and recurse. Future method queries see the replacement while an already returned old IMP remains callable. |

Blot remained smooth through the requested death test and exited normally.
Spoing remained smooth and exited normally. They are regression evidence, not
proof of every gameplay path. The log also contains a Glowfish HD launch with
the same pthread blocker even though the tester did not intentionally include
it.

## Implementation boundary

The changes are shared Objective-C runtime, Core Foundation, pthread and UIKit/
Core Graphics behavior. There are no bundle identifiers, game-specific
selectors, invalid-pointer allowances, swallowed exceptions, fabricated file
or service results, save changes, or diagnostic expansion. The universal
recorder remains enabled with its existing bounds.

`pthread_setcancelstate` and `pthread_setcanceltype` implement configuration
state only. The emulator still has no `pthread_cancel` delivery mechanism, so
this is deliberately a partial contract rather than a claim of full POSIX
cancellation support.

## Validation and predictions

Guest test sources cover runtime ivar lookup, method replacement and restoration,
canonical IANA lookup, cancellation defaults/old values/error handling, and
pattern tiling into a bitmap. Existing `class_getInstanceMethod` coverage also
calls a host-method IMP directly. Hosted library tests, guest executable tests,
iOS compilation and package verification are separate stages and must be
reported separately; the local Windows environment has no Rust/iOS toolchain.

Focused device order after a verified package:

1. Dark Nebula 2: enter the same level and play briefly if it starts.
2. Dirt: launch and continue as far as possible.
3. Glowfish, then Glowfish HD: launch and try to enter gameplay.
4. Hamlet: launch and continue as far as possible.
5. Parachute Ninja: launch and enter gameplay if possible.
6. Spoing and Blot: brief regressions; for Blot, play through one death.

Each removed blocker predicts only that execution advances beyond that exact
boundary. Menu, gameplay, audio, saving and functional compatibility remain
separate physical-device observations.
