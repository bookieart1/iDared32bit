# Dev 1.91: fifteenth-stage shared compatibility contracts

## Evidence

The two Dev 1.90 device logs embed source
`80c78363adaff869117b23e0ba15274dd973265e` and workflow
`41359428012`. They are archived outside the source checkout as
`outputs/device-logs/iDared-Dev-1.90-device-results-part-{1,2}.txt`:

- part 1: 132,878 bytes, SHA-256
  `18cbc1c2a70bb098c99f05c45de7fc515187e3d7866a879df5faf4aa8b60490f`;
- part 2: 332,303 bytes, SHA-256
  `d9b77d6c8bbd714212b699d04c908e84248e61fb30ede2e164f5608b4934cfd4`.

The universal bounded recorder was active for every eligible launch which
survived long enough to enter emulation. The combined evidence establishes:

- Cube now launches into gameplay and returns cleanly through emulator Home
  after 36.440 seconds, 2,153 main loops and 1,078 EAGL returns. It is playable
  in the observed scope, but bundled sounds such as
  `packages/sounds/free/shotgun.wav` fail the emulator's audio decoder. The log
  does not contain enough container metadata to identify a safe decoder change.
- Fragger repeatedly terminates the entire iDared host during startup. The
  surviving launch ends immediately after EAGL driver initialization, before a
  panic, Home event, progress report or recorder termination marker. This does
  not establish whether the origin is guest threading/audio, persisted state,
  an iOS host failure or an external kill; no speculative compatibility change
  is justified.
- Junny and Rolando independently stop before their first main loop when valid
  mutable strings receive
  `replaceOccurrencesOfString:withString:options:range:`. Both calls use the
  standard Foundation mutation path rather than a game-specific behavior.
- Toy Bot and Toy Bot 2 both reach menus and level-loading sequences, sustaining
  750/696 main loops and 249/231 EAGL returns respectively. Each then constructs
  an 88-byte `NSData` and calls `writeToFile:options:error:` with option 1
  (`NSDataWritingAtomic`) and a non-null error output.
- Spoing plays and returns through emulator Home normally after 8.066 seconds,
  208 main loops and 96 EAGL returns.

## Implementation

Dev 1.91 makes two reusable Foundation corrections and one bounded diagnostic
refinement:

1. Implement mutable-string occurrence replacement with UTF-16 ranges,
   non-overlapping matches, correct replacement count, case-insensitive,
   literal, backwards and anchored search behavior, and stable alias handling.
   Mutations are applied from the end so differing replacement lengths do not
   invalidate later ranges.
2. Implement `NSData`'s options/error write API. Atomic writes use a sibling
   auxiliary file and final filesystem rename; without-overwriting is honored;
   success leaves a supplied error pointer untouched, matching native
   Foundation; failure returns a Cocoa error with
   the historical file-path metadata. The older `writeToFile:atomically:` API
   reuses the same implementation instead of claiming atomicity while writing
   directly.
3. When an audio file cannot be decoded, extend the existing single failure
   line with bounded container metadata. For RIFF/WAVE this records format tag,
   channel count, sample rate, block alignment, bit depth and file size; it
   never logs audio contents. This is diagnostic-only and does not claim to fix
   Cube's sound.

There are no bundle-identifier checks, game-specific branches, fabricated
network/resource/audio success, relaxed object dispatch, weakened memory
checks, save resets or universal-recorder changes.

## Boundaries and risk

- Mutable-string matching reuses the emulator's existing NSString comparison
  machinery. Historical case-insensitive, literal, backwards and anchored
  options are handled; unsupported comparison option bits remain explicitly
  logged rather than silently acquiring invented semantics.
- Atomic data writing is synchronous and uses the emulator's existing guest
  filesystem. It provides replacement-by-rename on the iOS host and cleans up
  its auxiliary file after a reported failure. Filesystem failures remain
  failures and are not converted into successful saves.
- The generic Cocoa write error currently distinguishes an existing destination
  requested with `NSDataWritingWithoutOverwriting`; other host filesystem
  failures use the historical unknown-write error plus the failed path.
- Cube's audio defect remains unresolved until the next log identifies its WAV
  encoding. A missing sound is not reclassified as working audio.
- Fragger receives no compatibility patch. A device crash report or a surviving
  checkpoint around the first guest thread/audio transition is still required
  to distinguish its whole-host termination.

## Validation and predictions

Focused guest tests cover UTF-16 occurrence replacement, replacement counts,
case-insensitive matching, backwards overlap selection and write-failure path
metadata. The native Foundation fixture covers replacement and successful
atomic write/success-error preservation. A bounded audio-format test verifies
the new WAV metadata line.

Device predictions:

- Cube should remain playable in the observed scope. Its failed sound line must
  now expose the real container format metadata; this is evidence gathering,
  not an audio pass.
- Junny and Rolando must cross the mutable-string replacement call. Reaching a
  later blocker is progress, not a launch or gameplay claim.
- Toy Bot and Toy Bot 2 must cross the atomic save write and either enter
  gameplay or expose a later blocker. Existing saves must not be deleted.
- Spoing must retain normal launch/play/Home-return behavior.
- Fragger may be opened once to check reproducibility, but repeated whole-host
  crashes are not useful without an iOS crash report and are not required to
  validate the two compatibility contracts.

## Delivery status

Final source/package commit is
`34adad5822892507e593680040ebd14193eed4f5`; the main implementation commit is
`abcd130a5f021ab9062fd836371d5b34d4bebf72`, followed only by the native-contract
and Rust value-semantics correction found by the first hosted run.

- Hosted validation
  [41362430452](https://github.com/bookieart1/iDared32bit/actions/runs/41362430452)
  passed 166/166 development-library tests on both macOS and Windows. The
  native Foundation reference and both focused recorder reproducers passed on
  macOS. The macOS, Windows and Android lint commands each returned exit code 1
  for the development branch's existing long-comment diagnostics; those steps
  are explicitly allowed to fail. Formatting and the full guest-executable
  test stages were branch-gated and skipped.
- IPA workflow
  [41362430320](https://github.com/bookieart1/iDared32bit/actions/runs/41362430320)
  passed arm64 iOS compilation, separate identity verification, unsigned
  packaging and artifact upload. Artifact `13105961965` has outer ZIP SHA-256
  `c41d5aabf020895b7076a7579ca08b41e8f12aeb4f38da915582fff51612edf0`,
  matching GitHub's published digest.
- Local `outputs/iDared-Dev-1.91.ipa` is 20,051,553 bytes with SHA-256
  `75ea469db3881af5fdd57c281849cb78e1ee0b4181cb039b0dd4abd2ae8302ac`.
  Independent inspection verified its embedded source and workflow, arm64
  Mach-O, `com.bookieart1.idared32bit.dev` / `iDared Dev`, absence of a code
  signature and provisioning profile, permanent universal recorder, and all
  three new implementation/diagnostic markers.

Physical-device results remain separate from source, host-test and package
verification.
