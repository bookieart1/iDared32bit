# Dev 1.72: defaults removal and collection copying

October 31, 2026. This implements the first two opportunities in the complete
Dev 1.71 survey. It does not infer functional compatibility from removing a
blocker. Device results and artifact verification live in the compatibility ledger.

## Evidence and contract

The survey embeds `50ece578` (workflow 41250258745). MageGauntlet's history
shows `NSUserDefaults setObject:nil forKey:` reaching the strict dictionary
setter for `trpg/gcID`. HookWorld/QuickHook show the related `qh/uID` path, but
are from the same developer and are not three independent engine confirmations.
Infinight and three Sharpshooter launches instead dispatch `mutableCopy` on
valid mutable sets, then panic on the missing `mutableCopyWithZone:` selector.
These are two distinct contracts, not a single nil/object corruption family.

Fetched upstream `ca1de89c` still lacks both operations. The fork's preexisting
defaults KVO and reset/synchronize work must be preserved.

Primary references:

- [Foundation UserDefaults implementation](https://github.com/swiftlang/swift-corelibs-foundation/blob/main/Sources/Foundation/UserDefaults.swift): nil setting and removal both call CFPreferencesSetAppValue with nil.
- [Foundation NSSet implementation](https://github.com/swiftlang/swift-corelibs-foundation/blob/main/Sources/Foundation/NSSet.swift): mutable copying makes a separate mutable set; copying a mutable set makes an immutable snapshot.
- [Apple NSSet documentation](https://developer.apple.com/documentation/foundation/nsset): immutable/mutable collection distinction.

These modern references and the native fixture establish a defensible contract;
they are not execution on historical iPhone OS. An inspected 10.6 SDK header
declares both defaults operations but does not document nil semantics. Preserve
that evidence limitation rather than inventing a legacy validation result.

## Implementation boundary

- `ns_user_defaults.rs`: nil delegates to the existing removal method, including
  its existing observer notifications. Only the app domain is changed; fallback
  domains and unrelated saved values remain. Dictionary nil insertion still fails.
- `ns_set.rs`: both variants inherit mutableCopyWithZone; mutable sets implement
  copyWithZone as an immutable snapshot. Existing initWithArray owns retained
  elements, so object identities are shared but membership is independent.
- The same path uses allObjects: its old array neither retained its members nor
  autoreleased itself. Both concrete set variants now use a balanced snapshot
  helper. This directly supports copying and enumeration; it is not a general
  dictionary or mutation rewrite. Other set TODOs remain.
- `options.rs`, `environment.rs`, `OPTIONS_HELP.txt`: bounded flight diagnostics
  are enabled by default for every emulated app. The one-off survey switch is
  removed. Existing explicit off/flight overrides remain. No ring sizes, event
  filters, sampling rates or emulation responses are changed.

The user observed no noticeable slowdown in the universal survey and requested
this permanent default. This does not establish zero overhead for every game.
The dated diagnostic audit describes its original opt-in default; this record
supersedes that policy starting with Dev 1.72.

## Verification and exposure

Four added Rust library tests exercise real host-side Objective-C dispatch:
defaults domain removal/fallback and strict dictionary rejection; set identity,
mutability, isolation and enumeration; balanced element retention after draining
temporary pools and releasing source/copy in either order; default/override parsing.
A test-only Environment constructor provides thread-local autorelease storage.

`tests/FoundationCopyDefaults.m` is independently run against native macOS
Foundation by the development CI workflow. It checks nil-removal fallback,
one observer notification, exclusion from persisted defaults, strict dictionary
rejection, shallow copying and element lifetime. Native persistence is a contract
reference, not proof of emulator file persistence. No user saves are touched by
these fixtures. Host tests are not ARM guest ABI or physical gameplay tests.

Local source formatting and whitespace checks run before pushing. The local
offline test attempt stopped on an incomplete cached SDL submodule; the machine
also lacks the previously documented native C++ prerequisites. Actual hosted
test results and iOS compilation/package verification are recorded separately
in the ledger; the existing allowed-to-fail lint step is not proof of clean lint.

Expected device result: MageGauntlet passes the nil-default operation; Infinight
and Sharpshooter pass set copying without corrupting collection membership or
lifetimes. Later blockers remain possible. Check a short play/menu/relaunch
sequence for all three, with Spoing and Spirits as focused regressions including
existing saves. Do not reset or delete saves to obtain a passing result.
