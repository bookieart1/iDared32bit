# iDared Dev 1.82: sixth-stage contracts and host lifecycle

Date: November 1, 2026

## Evidence and implementation boundary

This iteration follows the complete Dev 1.81 device run. The unmodified
1,318,957-byte log is archived as
`outputs/device-logs/iDared-Dev-1.81-device-results.txt`; its SHA-256 is
`8a4407fd9473b898d0d14e72a0cf030446ffbc0747f0eb1cdeb841eb2773a48b`.
It embeds source `178cc5197ebc80415538d12aaa6458d3a3e47abc`, workflow
`41294836557`, and the universal bounded recorder.

Three supplied screenshots are preserved beside the log. Their SHA-256
hashes are:

- Glowfish HD portrait keyboard over a landscape guest:
  `3bcc91c2c9162c66d6e89129e255a60299500844dbc9740db19702e8b42ccbf3`.
- Hamlet's incorrectly oriented first presentation:
  `5e40d9e24b55da31059136d9033f51f980a63beb8e4ffda314016e7632095a6a`.
- Hamlet after the interface corrects itself:
  `1581624dfc0d191915fc6b5bf03d70836e286c8e1fe22c1db9e13b7b2dad6ac9`.

Dev 1.82 contains shared contract and host-integration corrections only. It
does not add game identifiers, suppress `abort`, invent missing resources,
tolerate invalid objects, change saves, or expand the recorder.

## Dev 1.81 device result

| App / surface | Observed result | First decisive evidence and classification |
|---|---|---|
| Native picker | Icons followed the finger during page swipes and both normal exits and crashes returned to the same page. | **Verified working in the requested scope.** The interactive paging and continuity work is retained unchanged. |
| Blot | Normal smooth play and clean emulator-Home return after 53.941 seconds, 2,132 loops and 2,101 EAGL returns. | **Regression pass in the stated scope.** |
| Dark Nebula 2 | The level-loading path advanced beyond raw ivar assignment, then failed after 18.949 seconds, 1,041 loops and 525 EAGL returns. | **Partial; verified later stage.** The first decisive blocker is the missing standard `+[NSMutableIndexSet indexSet]` class cluster entry point. |
| Dirt | Gravity now follows physical device rotation. A held movement touch continues toward the same edge after crossing into a black bar. Traversal and saving remain functional. | **Working checkpoint in a broader but still finite scope.** The original implementation accepted every black-bar touch, which could misroute unrelated games. Dev 1.82 captures only touches that begin inside the guest viewport; an already-captured drag remains clamped at the guest edge. |
| Glowfish | Startup advanced beyond `dispatch_once`, then failed after 1.259 seconds because `UIPasteboard` lacked `+pasteboardWithName:create:`. | **Startup-blocked at a newly precise shared UIKit contract.** The recorded call asks for a named board with `create:NO`; correct behavior is a nil result when no such board exists. |
| Glowfish HD | Menu, name entry, intro sequence and level selection were visible and responsive. Level loading later reported missing bundled `GameData/Levels/Level_01.gfl`, logged an error and called `abort` after 46.976 seconds, 3,117 loops and 2,268 EAGL returns. | **Major visible stage advance; partial.** The resource absence is not safely repairable without inspecting the actual package. Dev 1.82 deliberately leaves the genuine abort strict. |
| Hamlet | Runs after a visibly incorrect first orientation; the supplied screenshots show the first presentation rotated/cropped and the later menu correctly landscape. | **Working checkpoint with a verified host-orientation defect.** This is not treated as a Hamlet-specific layout problem. |
| Parachute Ninja | Still supplies `UIApplication` to guest code that sends it the error selector `domain`. | **Startup-blocked; unchanged invalid-object origin.** Dev 1.82 does not mask it by adding error APIs to UIApplication. |
| Spoing | The first launch hit a null-page fault at guest PC `0x000c2c70` after 4.254 seconds, 11 loops and 8 EAGL returns. Two immediate later launches worked and returned cleanly through Home. | **Working regression with one newly observed intermittent failure.** One inconsistent event is insufficient for a compatibility patch; retain as a focused regression watch. |

The user also established that entering the system screenshot review/save flow
while a guest is running closes the whole iDared host. This transition is not
present in the supplied log, but source inspection provides a strong, bounded
explanation: SDL's app-background event reaches `AppWillResignActive`, whose
handler calls `std::process::exit(0)` after notifying the guest. That is a host
lifecycle policy, not a guest crash.

## Dev 1.82 changes

### Foundation index sets

`NSIndexSet` and `NSMutableIndexSet` now use sorted, non-overlapping half-open
ranges. The implementation covers empty/index/range construction, membership,
intersection, first/last and neighboring-index queries, equality/hash,
immutable and mutable copies, and mutable add/remove operations. It does not
expand large ranges into individual indexes. Focused coverage exercises range
merge, split and independent immutable copying.

This is a general Foundation collection contract. It predicts that Dark
Nebula 2 will pass its exact recorded class-method blocker, but it does not
claim that the rest of level loading is compatible.

### Named pasteboards

`UIPasteboard` now exports the historical general/find names and implements a
bounded in-process registry for general, unique and named pasteboards.
`pasteboardWithName:create:` preserves the important creation distinction:
an absent board returns nil for `create:NO` and is created only for
`create:YES`. Stable identity, name, persistence, string/strings state,
change count and named-board removal are included with explicit ownership.

Arbitrary data types, item dictionaries, cross-process sharing and system
clipboard integration remain out of scope. This partial contract is sufficient
for Glowfish's recorded non-creating lookup without fabricating a board.

### Host scene orientation

The existing SDL orientation hint remains authoritative for the guest. On iOS,
the same transition now also asks the active `UIWindowScene` to adopt the
matching interface-orientation mask, using scene geometry updates on iOS 16+
and the historical rotation attempt on older systems. The landscape direction
uses the same guest/host inversion already required by touchHLE.

This targets native host UI layered over a landscape guest (notably the
software keyboard) and the first-frame scene geometry observed in Hamlet. It
does not change a game's own orientation declarations or fabricate an
orientation that the guest did not request.

### Direct-touch capture at letterbox bars

A mouse or finger must begin inside the visible guest viewport to become a
guest touch. Once captured, that same drag remains active and is clamped to the
nearest guest edge when it crosses a black bar. Touches beginning in a black
bar are ignored. Controller-generated virtual touches are unchanged.

This retains the verified Dirt behavior while removing the broad risk the user
identified: an unrelated game can no longer receive a new edge touch that
began entirely outside its displayed screen.

### Background/screenshot lifecycle

An app-background event now performs the same bounded guest save/termination
notification used by emulator Home and returns to the retained picker instead
of terminating the entire iDared process. A guest failure during that
notification is contained by the existing return-to-picker boundary.

Full suspend/resume is not claimed: audio, threads and run-loop suspension are
not yet modeled. The safe prediction is that accepting a screenshot no longer
closes the host application; the current guest session may end and return to
the picker as early iPhone OS applications did.

## Validation boundary and focused retest

Local source review and focused unit coverage are followed by the repository's
hosted library tests and iOS archive workflow. The development workflow allows
lint to fail and skips the guest executable integration path; a green job must
not be represented as coverage of those skipped paths. Swift scene integration
and physical screenshot behavior require the iPhone build.

Without deleting saves, test:

1. Blot: play briefly, then use emulator Home.
2. Dark Nebula 2: select a level and stop at the first failure.
3. Dirt: verify gravity, begin movement inside the game and drag into a black
   bar, then begin a fresh touch entirely in a black bar and confirm it does
   not control the character.
4. Glowfish: launch and try to reach gameplay.
5. Glowfish HD: reach level selection and start a level; if name entry appears,
   note the keyboard orientation.
6. Hamlet: record whether the first presentation is correctly landscape.
7. Parachute Ninja: launch once and allow the first error.
8. Spoing: launch twice; play briefly and use emulator Home each time.
9. In any stable running game, take a screenshot, accept/save it, reopen
   iDared, and report whether iDared stayed alive and returned to the picker.

Package verification and exact hosted-test results are recorded after the
workflow completes. They remain distinct from device compatibility.
