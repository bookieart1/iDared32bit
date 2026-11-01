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

## Verified delivery

- Final implementation/source commit:
  `d147f4fa707f4bdde98eea92f91c3bf981ada22f` (the compatibility contracts
  are in `e0c18280f63e9908d51f1ea5e58744750d80136e`; the final commit corrects
  only the synthetic guest-stack setup in the mutable-string unit test).
- Hosted test run `41352259787` passed: 155/155 development-library tests on
  macOS and 155/155 on Windows. The native Foundation reference, the two
  bounded diagnostic reproducers, and macOS/Windows/Android lint all passed.
  Formatting and the full release/guest-executable test steps were
  branch-gated and skipped.
- IPA workflow `41352259786` compiled and packaged the arm64 unsigned Dev app.
  Artifact ID `13102006128`; artifact ZIP SHA-256
  `553f24039296f28c9b0a02470091cb35c1d8c075f75ddfa4291f60f6d6fba787`,
  matching GitHub's published digest.
- Local `outputs/iDared-Dev-1.88.ipa`, 20,040,603 bytes, SHA-256
  `1cec5c66aea5276ee7496696d015f580df51849081e1a4a6dd3e02e21b6f9fe8`,
  passed arm64, development-bundle identity, unsigned/no-provisioning,
  embedded source/workflow, unchanged universal-recorder and new-contract
  marker checks.

Physical-device results remain pending and distinct from these build and
host-test checkpoints.
