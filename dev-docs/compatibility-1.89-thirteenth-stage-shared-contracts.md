# Dev 1.89: thirteenth-stage shared compatibility contracts

## Evidence

The Dev 1.88 device log embeds source `d147f4fa707f4bdde98eea92f91c3bf981ada22f`,
workflow `41352259786`, and the universal bounded flight recorder for all seven
tested launches. The log is archived outside the source checkout as
`outputs/device-logs/iDared-Dev-1.88-device-results.txt` (434,533 bytes,
SHA-256 `a8891741ff492a1f708c255edd8d85601cd439d8668d716b8a786c6da0162abc`).

Each failed title crossed its Dev 1.88 contract and exposed a different next
contract:

- Cube accepted all text drawing modes and then called the missing legacy
  `CGContextShowTextAtPoint`, immediately after `CGContextSelectFont` with
  MacRoman encoding. Its current drawing mode was invisible.
- Fragger passed the capacity-backed mutable-data path and reached the missing
  ARM Objective-C ABI entry point `objc_msgSendSuper2_stret` after 281 main
  loops and 282 EAGL returns.
- Junny and Rolando both reached `setValuesForKeysWithDictionary:` with a nil
  source. The emulator's newly added assertion, not guest corruption, caused
  both panics.
- Toy Bot and Toy Bot 2 both passed mutable-string insertion and independently
  sent `scanFloat:` to valid `NSScanner` instances during launch.
- Spoing played and returned through emulator Home, so it remains a healthy
  regression checkpoint.

## Implementation

Dev 1.89 makes four reusable corrections:

1. Export `objc_msgSendSuper2_stret` and implement the 32-bit ARM struct-return
   ABI. The hidden destination remains in r0; the `objc_super` receiver replaces
   r1 before normal superclass lookup. Host-originated `msg_super!` calls now use
   the same path for large structures.
2. Treat a nil argument to `setValuesForKeysWithDictionary:` as an empty source,
   matching normal Objective-C nil-message semantics. Non-nil dictionaries still
   use snapshot keys and ordinary `setValue:forKey:` dispatch.
3. Implement `NSScanner` decimal `scanFloat:` and `scanDouble:` with whitespace
   skipping, optional sign, decimal point and exponent, correct location updates,
   null-output support, and no output/location mutation on failure.
4. Implement legacy Core Graphics selected-font state and
   `CGContextShowTextAtPoint`. MacRoman bytes are decoded through the existing
   bounded encoding library; font-specific bytes are treated as glyph indexes;
   rendering reuses the existing glyph path. Invisible and clip-only modes do
   not rasterize.

The changes contain no bundle-identifier checks, fabricated network/resource
success, permissive memory behavior, or suppressed guest aborts. The universal
flight recorder is unchanged.

## Boundaries and risk

- Core Graphics stroke-only text and text clipping remain logged limitations.
  `CGContextShowTextAtPoint` does not yet expose or preserve a separately
  queryable current text position, which no current reproducer has requested.
- `NSScanner` locale-specific decimal separators and hexadecimal floating-point
  syntax are outside this first numeric contract. The implemented grammar covers
  the standard decimal path used by both Toy Bot titles.
- The super-dispatch change is ABI-sensitive but narrowly scoped to the missing
  struct-return entry point. A focused large-structure superclass test covers the
  hidden-pointer/register rewrite.
- Nil KVC input no longer trips an emulator-only assertion. Non-nil value
  validation and normal KVC errors remain strict.

## Validation and predictions

Focused tests cover superclass struct returns, nil bulk KVC, scanner success and
failure/location behavior, and MacRoman ASCII decoding. The native Foundation
fixture independently checks nil bulk KVC and scanner behavior. Hosted CI must
still be inspected for skipped or allowed-to-fail stages; successful compilation
is separate from device compatibility.

Device predictions:

- Cube must pass `_CGContextShowTextAtPoint`; reaching a later blocker is progress.
- Fragger must pass `_objc_msgSendSuper2_stret` when the tutorial starts.
- Junny and Rolando must pass the nil bulk-KVC call.
- Toy Bot and Toy Bot 2 must pass `scanFloat:`.
- Spoing must retain normal launch, gameplay and Home return behavior.

No title is promoted to working merely for crossing one of these contracts.
