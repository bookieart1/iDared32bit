# Dev 1.75: bounded dictionary provenance

October 31, 2026. This checkpoint follows the physical-device test whose log
embeds source `27825333` and workflow `41261997478`. The byte-identical log is
preserved outside the checkout as
`outputs/device-logs/iDared-Dev-1.74-device-results.txt` (228,943 bytes, SHA-256
`ee94bad4c631c3898d98e197397f5341718e8ef8a73b47d41bfe8fadaa245849`).

## What the device result establishes

- **Sharpshooter 1.8.1:** the 1.74 `NSMutableSet addObjectsFromArray:` repair
  removed the observed blocker. The user reports that the zoom interaction now
  works. The run continued for 56.367 seconds, with 4,991 main loops and 3,243
  successful EAGL presentations, and ended through the emulator Home control.
  This verifies progress through that interaction, not every game path.
- **Infinight 1.05:** the existing completed-level save was still present and
  the game remained clean in the tested scope. The log records a normal Home
  exit after 9.623 seconds, 561 main loops and 560 presentations. This is a
  regression pass for persistence and the collection changes.
- **Mage Gauntlet 1.30:** startup still fails at 163 ms, before a main loop or
  presentation. An immutable `_touchHLE_NSDictionary` receives
  `setObject:forKey:` at guest caller `0x1504d` after loading saved version 1.30
  and player state.
- **Hook World 1.10:** startup still fails at 144 ms when its Settings code
  mutates an immutable `_touchHLE_NSDictionary` at caller `0x74460`. The same
  preserved save loads version 1.10.

The 1.74 mutable singleton-factory hypothesis is therefore falsified for both
Rocketcat receivers. Do not extend it into a global rule that ordinary
`NSDictionary` objects are writable.

Hook World's earlier playable run was a first-run path: Dev 1.72 logged
`Loaded Version: (null)`, played for roughly 73 seconds, then failed while
saving/submitting the death score. Current launches load saved version 1.10
and fail earlier. Mage Gauntlet shows the same fresh-versus-restored distinction
(`(null)` in its earlier run, 1.30 now). This is evidence of a saved-state
stage change, not proof that a later source commit independently regressed a
previously working restored-state launch. The saves remain part of the test and
must not be deleted.

## Why this is diagnostic-only

Foundation and Core Foundation document returned preferences/property-list
values as immutable. In particular, `NSUserDefaults objectForKey:` and
`dictionaryForKey:` do not promise mutable returned containers, and Apple's
open-source CFPreferences implementation deep-copies property lists with the
immutable option. Making all restored dictionaries mutable would be a broad
semantic deviation with regression exposure across unrelated apps.

The remaining plausible origins include:

1. the games receive a nested immutable value directly from preferences and
   rely on behavior outside the documented contract;
2. the games requested a mutable copy but the emulator returned or propagated
   the wrong instance; or
3. the value came through keyed unarchiving or another collection factory/copy
   path whose mutability contract differs.

The existing 96-event ring proves the receiver class at failure, but the
producing boundary occurs too early and is overwritten. A compatibility change
cannot yet distinguish these explanations.

## Bounded provenance addition

The universal recorder now retains at most 128 live dictionary pointers and
the most recent selected object-producing boundary for each. Boundaries are
limited to preferences lookups, dictionary lookups, copy/mutable-copy,
property-list decoding, keyed decoding/unarchiving, and dictionary
initializers/factories. Non-dictionary results are rejected after inspecting
their concrete result class, so strings and numbers cannot churn the table.
An observed `dealloc` clears that pointer's entry to avoid reporting stale
provenance if an address is reused.

At a terminal dump, the recorder looks up the receiver of the newest unfinished
boundary and prints one matching producer/transfer event, if retained. It does
not call guest code, retain objects, modify ownership, change return values,
trace instructions, suppress the panic, or add normal per-frame output. The
ordinary 96-event history and every strict memory/object check remain intact.

Unit coverage verifies history-wrap survival, deallocation clearing,
non-dictionary filtering, and replacement by the newest transfer for the same
pointer. The recorder-size bound remains explicit. The enable marker now names
dictionary provenance so packaged-log coverage can be confirmed.

## Prediction and focused device test

The next Hook World and Mage Gauntlet panics should include either:

- a retained `objectForKey:`/`dictionaryForKey:` boundary, supporting a direct
  restored-preferences origin;
- a `mutableCopy` or mutable factory boundary, proving a concrete emulator
  mutability defect worth correcting; or
- a decoder/unarchiver boundary, narrowing the investigation to that contract.

No retained boundary is also useful: it rejects the selected paths and calls
for a different, still bounded origin probe rather than a speculative global
mutation workaround.

Test only the existing-save launch of **Hook World**, then **Mage Gauntlet**,
allowing each error to occur before returning to the picker. Sharpshooter and
Infinight do not need another full playthrough for a diagnostic-only change.

Local formatting and whitespace checks pass for the changed files. Local Rust
compilation cannot complete because the available Windows environment lacks
the MSVC linker; hosted macOS/Windows tests and iOS packaging must be recorded
separately. No physical-device result may be inferred from either.
