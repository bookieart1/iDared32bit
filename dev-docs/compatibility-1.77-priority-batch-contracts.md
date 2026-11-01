# Dev 1.77 priority-batch contract work

Date: November 1, 2026

This build targets the first user-prioritized batch from the Dev 1.71 catalog
survey: Parachute Ninja, Dark Nebula 2, Dirt, Hamlet and Glowfish. It retains
the universal bounded flight recorder and does not add app-specific branches.

## Evidence recovered

The complete `outputs/device-logs/iDared-Dev-1.71-survey.zip` was re-read before
editing. Its eight entries total 7,998,477 bytes and 53,967 lines; every entry
matches the dated extracted copy and survey manifest byte-for-byte. All eight
embed source `50ece57` and workflow `41250258745`. The relevant terminal
launches are:

- Parachute Ninja, `Log Page 4.txt` lines 6014-6194: `_sel_getUid` is missing
  before the first run-loop iteration or EAGL return.
- Dark Nebula 2, `Log Page 2.txt` lines 3352-3548: `_feraiseexcept(1)` is
  missing before the first run-loop iteration or EAGL return.
- Dirt, `Log Page 2.txt` lines 4159-4422: after GL/audio setup, `NSData` lacks
  `+dataWithContentsOfURL:options:error:`; the decoded option value is 2
  (`NSDataReadingUncached`).
- Hamlet, `Log Page 3.txt` lines 8928-9188: an `NSLog` path asks a valid
  `UIImage` instance for inherited `-description`, which `NSObject` lacks.
- Glowfish, `Log Page 3.txt` lines 6156-6708: anonymous `mmap` supplies
  `fd=0xf0000000`, the Darwin high-byte VM allocation tag, and hits the old
  `fd == -1` assertion. Glowfish HD repeats the same call and blocker at lines
  6709-7219.

These are five different contracts. Similar terminal panics elsewhere in the
survey were not treated as evidence that their origins were shared.

## Implemented boundaries

- Objective-C exports `sel_getUid` through the same selector interning path as
  `sel_registerName`, preserving pointer identity.
- The ARM CPU wrapper exposes the active thread's FPSCR to host calls.
  `feraiseexcept`, `feclearexcept` and `fetestexcept` update/read the real
  cumulative exception bits, so guest `VMRS` and fenv calls see one state.
  This does not add floating-point traps or revise the pre-existing rounding
  implementation.
- `NSData` implements the URL/file `options:error:` reader family for file
  URLs. Its defined options are mapping/cache hints; the guest filesystem's
  eager owned-byte read preserves visible bytes and lifetime. Missing files
  return `NSCocoaErrorDomain` / `NSFileReadNoSuchFileError`. Non-file URL
  loading remains at the existing unsupported network boundary.
- `NSObject` supplies the historical inherited instance-description shape
  `<Class: 0x...>` and `-debugDescription` dispatches through `-description`.
  Existing subclass overrides continue to win through normal dispatch.
- Anonymous `mmap` accepts Darwin `VM_MAKE_TAG` values only in the descriptor's
  documented high byte. Unrelated bits still fail with `EINVAL` before any
  allocation. `VM_FLAGS_PURGABLE` returns `ENOTSUP` because purgeable-memory
  lifecycle semantics are not implemented; it is not silently reported as
  supported.

Apple's Objective-C runtime header, archived NSData reference, iPhone OS mmap
manual and XNU VM tag definitions were used to settle these boundaries. The
current upstream touchHLE tree does not contain any of the five missing
contracts.

## Validation and prediction

Focused host tests cover selector identity, shared FPSCR exception state,
missing-file `NSError`, inherited descriptions without replacing an NSString
override, and accepted/rejected anonymous-mapping descriptors. Local
whitespace checking is available; this Windows workspace has no Rust toolchain,
so hosted compilation and tests remain required.

The build predicts only that each recorded first blocker is removed. Reaching
a menu or gameplay is not yet verified compatibility. Device testing should
launch the five games in the order above and exercise the first playable path,
then relaunch/save-check any game that becomes playable. Spoing and Blot are
the focused regressions: Spoing covers Unity/VM behavior and Blot covers
Foundation/UI transitions. Glowfish HD is an optional direct corroborator for
the VM-tag contract.

