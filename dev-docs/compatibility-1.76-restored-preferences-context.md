# Dev 1.76: bounded restored-preferences context

October 31, 2026. This diagnostic-only checkpoint follows the focused Dev
1.75 device test. The exact 116,002-byte device log is retained outside the
checkout as `outputs/device-logs/iDared-Dev-1.75-device-results.txt`, SHA-256
`19ead50def28badef62777dc67253a929022f0205762215069ef59da9a591a4c`.

## Evidence and remaining question

Dev 1.75 proved that the terminal immutable dictionaries in both Hook World
and Mage Gauntlet were returned directly by `NSUserDefaults objectForKey:`.
That rejects mutable-copy, mutable-factory and decoder/unarchiver origins for
those concrete receivers. It does not establish the preference key, selected
domain, or the key/value classes at the attempted mutation.

The missing context matters because Foundation's contract and Apple's
contemporary CF-550 implementation make loaded preference property lists
immutable. A blanket change that makes all defaults dictionaries mutable would
hide the crash by violating that contract and could regress unrelated games.
The next observation must distinguish a shared Rocketcat saved structure from
a broader persistence defect before any compatibility behavior changes.

## Implementation boundary

The universal flight recorder now retains a separate ring of at most eight
decoded boundary details. It records only:

- the string key for `NSUserDefaults objectForKey:` and `dictionaryForKey:`;
- the actual domain selected by the host implementation (`application`,
  `global`, `registration`, or `missing`); and
- for `setObject:forKey:` sent to the immutable concrete dictionary class, the
  value's concrete class and the key string/concrete class.

NSString text is copied through a bounded, allocation-free host-object helper
that supports the existing UTF-8, UTF-16 and UTF-32 backing forms. It never
sends a guest message, mutates or retains the observed object, changes a return
value, or follows an invalid pointer. Control characters remain escaped by the
existing recorder printer. The decoded key/domain are carried into the
existing 128-entry dictionary provenance table, so they survive ordinary
96-event history wrap and appear beside the terminal receiver's producer.
The retained producer key uses a compact 48-byte representation. This keeps
the complete recorder under its existing 128 KiB ceiling rather than expanding
the recorder to accommodate this diagnostic.

No preference value, collection mutability, ownership rule, persistence file,
memory check, graphics/audio behavior, title-specific path, or save is changed.
Universal recording remains the default for every eligible app.

## Validation and prediction

Unit coverage checks bounded UTF conversion without changing its backing form,
the domain/key association for a real defaults/dictionary object, decoded
immutable mutation arguments, and the recorder's existing 128 KiB size bound.
The full hosted library suite and iOS package build remain required because the
local Windows environment has no Rust toolchain available. Whitespace checking
passes locally.

On the device, an existing-save launch of Hook World followed by Mage Gauntlet
should still fail at the same strict immutable-dictionary checks. Each terminal
dump should now identify the defaults key/domain that produced the receiver and
the failing mutation's key/value classes. A visible compatibility improvement
is neither expected nor claimed.

If both apps name the same saved structure and compatible mutation shape, the
result supports investigating one shared Rocketcat legacy behavior. If the
keys or shapes differ, the next work should remain separated. Missing decoded
context would mean this probe failed and must be corrected before attempting a
compatibility accommodation. Existing saves must not be deleted.

## Verified build

- Final implementation and delivered source commit:
  `c5005dd4bb636f211c2c49d17b564a9f728dfd10`. Earlier commits `f97b6d73`
  and `f728c982` were superseded after hosted compilation exposed fixture-type
  errors and then the recorder-size ceiling; neither is the delivered build.
- [Hosted tests 41267274637](https://github.com/bookieart1/iDared32bit/actions/runs/41267274637):
  **122/122 library tests passed on macOS and 122/122 on Windows**. The native
  Foundation reference, decoded defaults/mutation test, recorder-size ceiling
  and existing diagnostic reproducers passed. Guest integration remains a
  configured skip.
- The Android lane stopped at its allowed lint failure on the same two
  pre-existing long comment lines in `dlfcn.rs` and `ns_dictionary.rs`.
  Clippy, rustdoc and Android compilation were therefore not reached.
- [IPA build 41267274701](https://github.com/bookieart1/iDared32bit/actions/runs/41267274701)
  succeeded. Artifact `13068265913`; the downloaded artifact ZIP SHA-256
  `ecaaff71d204daeac6af6d7c6f4197cd5215917e3f11a7742afd46dea9e9cc01`
  matches GitHub's published digest.
- Local `outputs/iDared-Dev-1.76.ipa`, **19,892,219 bytes**, SHA-256
  `3ce39870c3e3ca4f23da449fe6cd1e54f23adf4cf1b5766066ff9cff822774dd`,
  passed archive extraction/CRC and package-identity checks. Its executable is
  arm64, embeds source `c5005dd4`, workflow `41267274701`, branch/repository,
  the new decoded-context marker and the universal-flight default. The package
  is unsigned under `com.bookieart1.idared32bit.dev` / `iDared Dev` and has no
  embedded provisioning profile or `_CodeSignature` directory.
- This remains diagnostic-only. Physical-device validation is recorded below.

Verification record: `outputs/iDared-Dev-1.76.verification.json`.

## Physical-device result

The November 1 focused test used the delivered source `c5005dd4` from workflow
`41267274701` and launched existing-save Hook World followed by Mage Gauntlet.
The exact 117,729-byte log is retained as
`outputs/device-logs/iDared-Dev-1.76-device-results.txt`, SHA-256
`38d5a42c3093e9e000f7d7204511b6e9e0527dd69c21e78e94a012553a95e9b4`.

The diagnostic succeeded and neither title changed behavior:

- Hook World received immutable dictionary `0x3647e150` directly from
  application-domain `NSUserDefaults objectForKey:` for `qh/items`. It then
  attempted `setObject:forKey:` on that exact receiver with an `NSNumber`
  value and key `player/zelle` (guest caller LR `0x74460`).
- Mage Gauntlet received immutable dictionary `0x3646d700` directly from
  application-domain `NSUserDefaults objectForKey:` for `trpg/stats`. It then
  attempted `setObject:forKey:` on that exact receiver with an `NSString`
  value and key `firstPlay` (guest caller LR `0x1504d`).

This rejects the narrow hypothesis that both failures are one identical saved
key or one corrupt shared object. It supports a shared Rocketcat Settings
pattern: two independently named preference dictionaries are both mutated
directly after being restored as immutable property-list values. The concrete
receivers did not originate from an intervening mutable copy.

The result does not justify making all `NSUserDefaults` dictionaries mutable.
Apple's historical CFPreferences contract says returned property-list values
must be assumed immutable, and the contemporary implementation deep-copies
preference values as immutable. The emulator also has a separate general
correctness issue: its current `setObject:forKey:` stores the supplied object
by reference, so a newly created mutable nested dictionary can remain mutable
within its creation session; serialization and relaunch then expose the strict
immutable form. Correcting that snapshot behavior would improve Foundation
semantics but would not make these direct post-relaunch mutations valid and
could expose them earlier.

No further logging is needed to identify this boundary. Before a compatibility
change, inspect the shared Rocketcat Settings call sites and early-iOS behavior
for evidence of an app-side mutable-copy expectation. If a compatibility
accommodation is ultimately necessary, keep it explicit and narrowly scoped
rather than weakening the global preferences contract.
