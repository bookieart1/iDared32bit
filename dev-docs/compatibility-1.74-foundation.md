# Dev 1.74: mutable collection contracts after the 1.73 device test

October 31, 2026. This checkpoint is based on the physical-device log whose
banner embeds source `452a8571` and workflow `41260610289`, not on an archive
label. The unmodified log is preserved as
`outputs/device-logs/iDared-Dev-1.73-device-results.txt` with SHA-256
`9076db46ad3291f05a4753b9cd45fe322ccdfd2e404f9d323948272aa7746ba`.

## Device evidence

- **Mage Gauntlet 1.30** fails twice during restored-player/settings startup.
  In each launch a concrete immutable `_touchHLE_NSDictionary` receives
  `setObject:forKey:` at guest caller `0x1504d`. The failure occurs before a
  main loop or presentation (`174/126 ms`, zero EAGL returns). The recorder
  proves the receiver's class at the mutation site, but its bounded history
  does not retain the object's older construction.
- **Sharpshooter 1.8.1** reaches the user's initial gameplay scene. After
  18.138 seconds, 1,746 main loops and 968 EAGL returns, a genuinely mutable
  `_touchHLE_NSMutableSet` receives the standard missing method
  `addObjectsFromArray:` with an array argument. This is a decisive API
  blocker, not evidence of corrupt object identity.
- **HookWorld 1.10** fails twice while loading and extending its Settings item
  dictionary. An immutable `_touchHLE_NSDictionary` receives
  `setObject:forKey:` at caller `0x74460` after repeated successful item
  lookups. Each launch fails around 140 ms with one EAGL return. Dev 1.72 had
  reached playable HookWorld before writing progress; the changed stage is
  consistent with a save/reload-dependent path. It is not enough evidence to
  classify Dev 1.73 as a general launch regression.

All five CPU launches have universal bounded recorder coverage and terminal
dumps. The diagnostic default remains unchanged. The user reported the visible
progress boundaries; the logs alone cannot establish the exact frame content.

## Contract corrections

1. Implement `-[NSMutableSet addObjectsFromArray:]` by enumerating the supplied
   array and using the existing `addObject:` ownership and uniqueness path.
   This is a complete standard collection operation within the emulator's
   existing array/set boundaries.
2. Correct `+[NSMutableDictionary dictionaryWithObject:forKey:]`. The inherited
   convenience method always constructed the private immutable dictionary,
   even when invoked on the mutable class. The mutable class now creates its
   mutable concrete implementation while the NSDictionary factory remains
   immutable. Nil input remains rejected.
3. Add host tests for duplicate set input, membership, factory class identity,
   post-factory mutation, and keyed-archive round-trip mutability. Extend the
   independent native Foundation fixture with the same contracts.

The dictionary correction is a real general semantic bug, and a credible
explanation for the two Rocketcat failures, but the recorder did not capture
the actual creation of either failing object. Therefore its app-level effect is
a prediction, not a verified root-cause claim. If Mage Gauntlet or HookWorld
still receives an immutable saved child, the next investigation must trace that
specific object's archive/property-list origin. It must not make ordinary
NSDictionary instances writable or deep-mutate property-list children: Apple's
property-list convenience contract does not promise mutable nested containers.

No game-specific branch, save migration, nil acceptance, memory-fault
suppression, rendering change, audio change or diagnostic expansion is part of
this checkpoint. Existing development saves must be preserved for the device
test because the saved-state path is part of the evidence.

## Prediction and validation plan

- Sharpshooter should pass the exact `addObjectsFromArray:` blocker and either
  continue gameplay or expose a later independent failure.
- Mage Gauntlet and HookWorld will pass their current mutation only if the
  offending dictionary originated from the corrected mutable singleton
  factory or a correctly preserved mutable keyed archive. Continued immutable
  receivers would falsify that narrow origin hypothesis.
- Focused device order: Mage Gauntlet launch/start; Sharpshooter reach gameplay
  and repeat the crashing interaction; HookWorld launch, play to death if it
  opens, then relaunch with the same save. Infinight is the closest collection
  regression control and should receive a brief old-progress/relaunch check.

Local Rust execution is unavailable in this workspace because no Rust toolchain
is installed. Changed-file whitespace review is clean. Hosted library tests,
native fixture execution, iOS compilation and package verification must be
recorded separately after the workflows finish.
