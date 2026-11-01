# Dev 1.73: singleton arrays, audio URL, and HookWorld crash evidence

October 31, 2026. Based on the physical-device log embedding source `05732a4`
and workflow `41258098358`, not its filename. SHA256 of the unmodified log:
`12f537252f911e82c4a2f6cc281c4f38a8868acd6a72199d96d9377a246bc0ec`.
Local evidence: `outputs/device-logs/iDared-Dev-1.72-device-results.txt`.

## Verified device findings

- **Infinight 1.05**: user completed a level, returned to menu, exited and
  reopened with progress retained. Both launches end through emulator Home,
  with 4,238 and 577 EAGL returns respectively. Functional in that tested scope;
  deeper play and every audio/input path are not certified.
- **MageGauntlet 1.30**: two presentations then fails in GDataXML children.
  Recorder call 7515871 sends `arrayWithObject:` to **NSMutableArray**. The
  implementation allocates **NSArray**, returning `0x366483a0`. Call 7515899
  appends a second child and panics on missing `addObject:`. This is a wrong
  factory result, not an invalid guest mutation or arbitrary dispatch fault.
- **Sharpshooter 1.8.1**: 1,103 EAGL returns, user reaches the transition into
  gameplay. `SniperAudioController startNextTrack` stops an existing player,
  then queries its `url`; that valid AVAudioPlayer lacks the getter. The URL
  already exists in its retained host state. No evidence of a decoder fault.
- **HookWorld 1.10**: playable until the recorded avalanche death, 4,307 EAGL
  returns. It saves score 552, decodes local-best LevelStats and enters
  `FeintScore scoreFromLevelStats:PlayerEquipment:` at caller `0x972dc` with
  R3=`0x36467370`, the **_touchHLE_NSString class**, as equipment. `playerChar`
  subsequently fails at caller `0x709c0`. This does not justify adding a
  playerChar selector to NSString or bypassing score handling.

All five CPU launches have recorder coverage and a complete terminal dump.
The prior defaults/set-copy blockers are absent on these exercised paths.
HookWorld/Mage/Sharpshooter advanced to different failures; this is not proof
of full compatibility or a demonstrated regression. Spoing/Spirits were not
tested in this log. QuickHook was not the Hook game tested.

## Changes and contract boundary

1. `ns_array.rs`: add the mutable singleton factory override, using existing
   capacity construction and addObject ownership. Keep immutable singleton
   construction immutable. Reject nil singletons on both classes. This follows
   the class-cluster override pattern already used by other mutable factories.
   No mutation method is added to immutable arrays.
2. `av_audio_player.rs`: expose the existing retained URL without changing
   ownership. Data-backed players return nil. Stop already preserves the URL;
   no new audio state, backend, callback or playback-success behavior is added.
3. `diagnostics.rs`: at a panic only, read at most eight distinct recent caller
   locations from the existing bounded recorder history on the failing thread.
   Print up to 112 bytes around each, constrained to a known loaded image and
   using fallible reads. Preserve LR's Thumb bit. No extra per-call/per-frame
   collection, larger ring, continuous CPU tracing, guest method calls, or
   memory writes. Home exits do not emit these windows; duplicate dumps do not
   emit them again. These are recent call sites, **not a reconstructed stack**.

The two compatibility changes are general API corrections. HookWorld receives
**diagnostic-only** investigation in this build, not a claimed death-crash fix.
Its wrong equipment argument precedes the final selector failure. Existing
null-memory fault windows do not run for this selector panic, and the history
does not retain the producer. The new windows should expose the instructions
near `0x972dc` that supplied R3, separating direct field/global loads from an
earlier returned value. If that still does not establish origin, use the
matching executable/save data rather than guessing broader runtime semantics.

Freshly fetched upstream `ca1de89c` has the same missing mutable factory and
audio getter. No upstream repair was available to reuse. Relevant references:

- [Apple NSArray](https://developer.apple.com/documentation/foundation/nsarray): immutable arrays and their mutable subclass.
- [Apple AVAudioPlayer URL](https://developer.apple.com/documentation/avfaudio/avaudioplayer/url): source URL, nil for a player not created with a URL.
- Native macOS Foundation fixture independently checks the singleton mutability
  and nil-rejection contract. It is not a historical iPhone OS runtime oracle.

During HookWorld investigation, NSKeyedUnarchiver's handling of UID zero and
convenience-method ownership warrant a separate round-trip audit. Neither is
established as the cause of this class-valued equipment argument. No coder,
OpenFeint, ABI, saved-data or object-lifetime change is made on that hypothesis.

## Validation and next device test

New host tests exercise singleton mutability/append/identity/refcounts/pool
drain and nil rejection, audio URL borrowed ownership after stop and a real
WAV-backed data initializer returning nil URL. Recorder tests cover wraparound,
deduplication, thread selection, fixed capacity, unchanged missing-selector
panic with diagnostics off/on, unchanged registers/CPSR/guest bytes after dump,
and a synthetic A32 caller window. This does not execute the actual game code.
The macOS native reference fixture is extended for singleton arrays. Hosted
compilation/test and package results will be recorded in the ledger after CI.
Local changed-file formatting and whitespace checks pass; the previously
recorded missing local native build dependencies remain, so CI is required.

Device plan: MageGauntlet start/play; Sharpshooter start/play and return to menu;
HookWorld play until death/retry; brief Infinight old-progress/relaunch control.
Install over the existing Dev app and preserve all saves. Universal recording
remains the permanent default. A subsequent blocker is progress, not yet proof
that a game is fixed. Infinight's existing successful result must not be lost.


## Completed build validation

Source `452a8571`, IPA workflow 41260610289, test workflow 41260610296.
116/116 library tests pass on each host; native Foundation and both diagnostic
reproducers pass. An initial test-only missing guest stack was corrected before
delivery. Actual synthetic output contains the expected A32 caller bytes once.
The eight-caller selection includes the desired HookWorld callers in the
provided history. iOS archive and local package identity/checksums verified.
Existing lint limitations and exact package hashes are recorded in the ledger.
Game behavior on this build is still awaiting the next physical-device test.
