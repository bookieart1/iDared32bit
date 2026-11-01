# Dev 1.90: fourteenth-stage shared compatibility contracts

## Evidence

The two Dev 1.89 device logs embed source
`73746bd167835c87fd47d96505f7e3424e1dd95c` and workflow
`41355012675`. They are archived outside the source checkout as
`outputs/device-logs/iDared-Dev-1.89-device-results-part-{1,2}.txt`:

- part 1: 109,250 bytes, SHA-256
  `101c53a905a1a13d01a44263f09a16ace9524289ad7270daaafb8b046cb9b9ba`;
- part 2: 330,711 bytes, SHA-256
  `a47a232295bad8eb99630f1bfdd11baead0a74082e839b77de2620031cddb24d`.

The universal bounded recorder was active for every eligible launch. The two
halves establish four distinct reusable blockers:

- Cube passes `CGContextShowTextAtPoint` in invisible mode, then immediately
  calls the missing `CGContextGetTextPosition`. The call sequence is the
  historical measure-with-invisible-text pattern: select font, set an inverted
  text matrix, select mode 3, show one byte, then read the advanced position.
- Fragger runs its tutorial for 44.427 seconds (2,604 main loops and 2,604 EAGL
  returns), logs `Score save`, and calls
  `+[NSPropertyListSerialization dataFromPropertyList:format:errorDescription:]`
  with binary format 200 and a non-null error-output pointer. The emulator's
  assertion that this standard output pointer must be null is the decisive
  failure.
- Junny and Rolando independently send `copyWithZone:` to the valid `NGGame`
  class object during launch. Both stop at the same missing inherited
  class-side method before a main loop.
- Toy Bot and Toy Bot 2 both reach their menu and level-loading animation, then
  stop at the same `printf` assertion for the standard alternative form `%#e`.
  Each had already sustained hundreds of main loops and presented frames.

The second Fragger launch ends the entire supplied log during startup without
an emulator panic, Home event, or flight-recorder termination marker. That is
evidence of an unexplained host termination, not enough evidence for a safe
compatibility change. It remains a separate observation to retest after the
property-list failure no longer interrupts the first run.

## Implementation

Dev 1.90 makes four general corrections:

1. Track the current Core Graphics text position, export its getter and setter,
   and advance the position after legacy at-point text/glyph display. Advance
   calculation reuses the rasterizer's scale and pair-kerning rules and applies
   the linear text transform. Invisible and clip-only modes still do not
   rasterize, but they now update the position as required for measurement.
2. Accept the standard non-null `NSString **` output in property-list
   serialization and clear it to nil on success. Successful parsing likewise
   clears a supplied error output. Unsupported formats and invalid values
   remain strict; serialization failures are not converted into fabricated
   data.
3. Supply NSObject's inherited class-side `copyWithZone:` contract. Class
   objects are immutable singletons, so the method returns the same class
   object without retain-count side effects.
4. Implement `#` for the supported `%e`, `%f`, and `%g` conversions. The
   alternative form preserves a decimal point at zero fractional precision;
   all existing width, zero-padding, sign, precision, and exponent handling is
   retained.

No bundle-identifier checks, game-specific branches, permissive object or
memory behavior, fabricated resource/network success, or diagnostic changes
are included. The universal flight recorder remains enabled unchanged.

## Boundaries and risk

- The Core Graphics change models the position/advance contract used by Cube.
  Character-spacing state, stroke-only rasterization, and text clipping remain
  existing limitations. The text matrix remains a context attribute and is not
  incorrectly pushed onto the graphics-state stack.
- Property-list serialization still supports binary output only and the
  emulator's existing bounded property-list types. This change corrects the
  success-output contract; it does not swallow invalid property-list values.
- Class-side copying is inherited from NSObject and returns the receiver, as
  required for class objects used as collection keys. It does not add a generic
  instance `copyWithZone:` implementation.
- The floating-point change covers only conversions the formatter already
  implements. Unsupported integer alternative forms remain explicit failures.
- Fragger's truncated second launch remains unresolved until a post-fix run can
  distinguish persistent state, host lifecycle, and an unrelated later fault.

## Validation and predictions

Focused tests cover transformed text advance, successful property-list error
output clearing, inherited class-object copying, and alternative-form decimal
points with padding/sign behavior. The native Foundation fixture separately
checks class copying and property-list success-output semantics, while the guest
CLI formatter fixture checks Darwin `%#.0e` and `%#.0f` results.

Device predictions:

- Cube must pass `_CGContextGetTextPosition`; reaching a later blocker is
  progress, not a gameplay claim.
- Fragger must pass the score-save serialization after its tutorial/level run.
  A clean Home/relaunch then tests the previously unexplained shutdown.
- Junny and Rolando must pass the class-object `copyWithZone:` call.
- Toy Bot and Toy Bot 2 must pass `%#e` during level loading and either enter
  gameplay or expose a later blocker.
- Spoing must retain normal launch, play, and Home-return behavior.

No title is promoted to working merely for crossing one contract.

## Verified delivery

The delivered source/package revision is
`80c78363adaff869117b23e0ba15274dd973265e`.

- Hosted test run
  [41359428038](https://github.com/bookieart1/iDared32bit/actions/runs/41359428038)
  passed 162/162 development-library tests on both macOS and Windows. The
  native Foundation reference and the focused diagnostic reproducers passed on
  macOS. Lint emitted exit code 1 on macOS, Windows and Android but is allowed
  to fail on this development branch. Formatting and full guest-executable
  tests were branch-gated and skipped.
- IPA run
  [41359428012](https://github.com/bookieart1/iDared32bit/actions/runs/41359428012)
  produced artifact `13105100103`. The artifact ZIP SHA-256 is
  `c805e4e3025033a2ecb0fbf9e7842afafd66f414e3be02c73a8b4a7d9e05f9b6`,
  matching GitHub's published digest.
- The verified local IPA is 20,054,430 bytes with SHA-256
  `f38857c82ee134758c3639ac6bd3faffc1a32be1685c6ba0ff849f988e513acc`.
  It is an unsigned arm64 package with no provisioning profile, bundle ID
  `com.bookieart1.idared32bit.dev`, display name `iDared Dev`, embedded source
  and workflow identifiers, and the unchanged universal recorder.

Physical-device results remain pending and separate from build verification.
