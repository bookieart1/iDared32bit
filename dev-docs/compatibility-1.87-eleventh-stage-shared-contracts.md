# Dev 1.87: eleventh-stage shared compatibility contracts

## Evidence and scope

The Dev 1.86 device log embeds implementation `9b9e2df984d7390b2e39ca90e55fc3f8e59e2696`, IPA workflow `41313333541`, and the universal bounded flight recorder. The recorder remained active for every decrypted app that entered CPU emulation.

Verified Dev 1.86 observations:

- Blot, Dark Nebula 2, Dirt, Crazy Golf, and Spoing remain functional in the user's tested scopes. Crazy Golf now renders and operates its sliders.
- Cube stops at `CGFontCopyPostScriptName` after successfully creating and inspecting a valid `CGFont`.
- Fragger advances through level-pack selection and then asks `mutableBytes` of a valid zero-length `NSMutableData`. Empty mutable data is legal; the emulator's non-empty assertion is wrong.
- Glowfish completes gameplay and returns to its menu, but its new persisted state then calls missing `atoll`; later launches reach the same blocker early.
- Junny passes the prior table-cell contract and next allocates `NSNumberFormatter`.
- Rolando passes subclass-safe bundle initialization and next allocates `NSCountedSet`.
- Toy Bot and Toy Bot 2 pass the prior `UITextView` keyboard-type state and next set the standard keyboard-appearance trait on a valid text view.

The Crazy Golf Home path separately exposes a property-list serialization panic because an `UIImage` is present in its defaults domain. The game already works, and the log does not establish where the non-property-list object originated. Dev 1.87 does not suppress that panic or invent an image serialization.

## General contracts implemented

- `CGFontCopyPostScriptName` returns an owned string. Fonts loaded through a data provider use their OpenType name table; fonts requested by a historical public name preserve the guest-visible name when touchHLE substitutes a host font.
- `NSMutableData.mutableBytes` permits a zero-length receiver and returns its current pointer, which may be null. Callers remain responsible for respecting `length`.
- Darwin `atoll` performs signed 64-bit decimal conversion rather than truncating through the guest's 32-bit `long`.
- `NSNumberFormatter` gains bounded historical allocation/state plus decimal, currency, percent, and scientific string conversion. This is a partial formatter contract, not a claim of locale-complete modern Foundation behavior.
- `NSCountedSet` stores retained distinct members and their multiplicities, including standard membership, enumeration, add/remove, and teardown behavior.
- `UITextView.keyboardAppearance` is stored and reported just like the already implemented keyboard-type trait. This does not itself change host keyboard orientation or styling.

No app identifier, save reset, unavailable-service success, resource fabrication, invalid-object tolerance, swallowed abort, or weakened memory check is included. The universal recorder is unchanged.

## Predictions and focused validation

- Cube should pass font-name inspection.
- Fragger should pass tutorial-level image-buffer creation.
- Glowfish should pass both the post-game replay path and relaunch with its existing save.
- Junny and Rolando should pass their newly identified Foundation class-allocation blockers.
- Toy Bot 1 and 2 should pass text-view keyboard appearance setup.

The next failure, if any, is evidence for another contract and not proof that a title is fully compatible. Crazy Golf and Spoing are the most relevant regressions for this set. Dirt is promoted to a working checkpoint and is removed from routine testing unless a future change touches motion, edge input, saving, or its engine family.

## Deferred issues

- Glowfish HD: the IPA lacks `GameData/Levels/Level_01.gfl`; the guest then deliberately aborts. Its portrait host keyboard in landscape remains a separate orientation problem.
- Hamlet: the rotated/cropped opening remains a presentation-orientation defect.
- Parachute Ninja: an unavailable Plus+ path supplies a real `UIApplication` where later code expects an error object and sends `domain`. The invalid value's origin must be found; `UIApplication` will not receive fake error methods.
- Crazy Golf: the defaults-domain `UIImage` origin needs bounded creation/insertion evidence before changing property-list behavior.

## Validation boundaries

Local Rust compilation is unavailable in the current Windows host environment. The pushed implementation must therefore pass the repository's macOS and Windows development-library jobs, focused guest ABI tests where available, and the separate arm64 iOS packaging workflow before an IPA is delivered. A green overall badge is not sufficient: skipped and allowed-to-fail steps must be inspected separately.
