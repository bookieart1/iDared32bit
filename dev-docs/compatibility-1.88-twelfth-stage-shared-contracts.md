# Dev 1.88: twelfth-stage shared compatibility contracts

## Device evidence and scope

The complete Dev 1.87 device log is archived outside the source checkout as
`outputs/device-logs/iDared-Dev-1.87-device-results.txt`: 806,996 bytes,
SHA-256 `344945012f4583b6a325ec0b60ea3d3509f59d68619d6aaecd6c7904cff393f`.
It embeds implementation `58fd8badf1eb210f2a697a7402266af4d3a8a3f0`, IPA workflow
`41319727039`, and the unchanged universal bounded flight recorder.

Verified observations:

- Crazy Golf's sliders and gameplay work. Its emulator-Home cleanup still
  finds a `UIImage` in property-list data and catches a serialization panic
  while returning to the picker. This is not a gameplay failure, and the log
  still does not establish the invalid value's insertion origin.
- Cube passes font creation and PostScript-name inspection. It next calls
  `CGContextSetTextDrawingMode` with the valid historical value 3
  (`kCGTextInvisible`), which the emulator incorrectly rejects.
- Fragger passes level-pack selection and tutorial setup. Its Cocos2d render
  texture allocates `NSMutableData` with capacity `0x96000`, receives a null
  `mutableBytes` pointer, and passes it to `glReadPixels`, producing the exact
  null-page write. The recorder identifies
  `-[CCRenderTexture getUIImageFromBuffer]` as the guest caller.
- Glowfish retains progress, enters gameplay, pauses, returns to the menu and
  re-enters gameplay without crashing. It is a working checkpoint in this
  tested scope rather than proof that every level is defect-free.
- Junny passes `NSNumberFormatter` allocation and stops at the standard
  `setPositiveFormat:` pattern property.
- Rolando passes `NSCountedSet` and stops when a valid mutable dictionary
  receives standard KVC `setValuesForKeysWithDictionary:`.
- Spoing remains a clean regression pass.
- Toy Bot and Toy Bot 2 pass text-view keyboard traits and stop at the same
  standard UTF-16 mutable-string insertion method,
  `insertString:atIndex:`.

## General contracts implemented

- Core Graphics stores all eight historical text drawing modes in graphics
  state. Invisible and clip-only modes do not paint pixels. Existing fill
  behavior is retained for modes with a fill component. Stroke rasterization
  and text-generated clipping remain explicit limitations rather than being
  falsely claimed.
- `NSMutableData` tracks allocated capacity separately from logical length.
  `initWithCapacity:` now provides retained writable storage, growth reuses it
  when possible, newly exposed logical bytes are zeroed, and the storage is
  released through the existing ownership path. This preserves strict memory
  faults; it does not make the null page writable.
- `NSNumberFormatter` copies and owns positive and negative format strings and
  applies a bounded historical decimal-pattern subset: `0`, `#`, decimal,
  grouping, percent, and literal prefix/suffix behavior. This is not a claim of
  locale-complete ICU formatting.
- NSObject KVC implements `setValuesForKeysWithDictionary:` through the
  receiver's normal `setValue:forKey:` dispatch using a key snapshot. This
  preserves subclass setters and dictionary overrides.
- `NSMutableString` implements `insertString:atIndex:` through the primitive
  `replaceCharactersInRange:withString:` contract. The concrete mutable string
  validates ranges and mutates in UTF-16 code-unit space, including surrogate
  pairs.

No game identifier, unavailable-service success, resource fabrication,
invalid-object tolerance, swallowed abort, writable null page, or weakened
memory check is included. The universal flight recorder is unchanged.

## Predictions and focused device validation

- Cube should pass invisible-text setup.
- Fragger should pass the Cocos2d screenshot/readback buffer creation used by
  the tutorial-level transition.
- Junny should pass positive number-format setup.
- Rolando should pass bulk dictionary/KVC assignment.
- Toy Bot 1 and 2 should pass mutable-string insertion.
- Spoing is the focused broad regression.

The next failure is evidence for another contract and is not itself a claim of
full compatibility. Crazy Golf and Glowfish are removed from this routine
retest: both are working checkpoints in the observed gameplay scope. Crazy
Golf's separate cleanup/save-origin question remains recorded.

## Validation boundary

Local Rust tooling is unavailable in the current Windows environment. Hosted
macOS and Windows development-library tests and the separate arm64 iOS
packaging workflow must pass before delivery. Skipped and allowed-to-fail
workflow stages must be reported separately from actual tests.
