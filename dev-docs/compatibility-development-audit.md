# Compatibility development audit and flight recorder v1

Audit date: 2026-10-31. Baseline: `c9c3f765` (device-tested Dev 1.65).
This is a diagnostic-only change. Device validation of Dev 1.66 is pending.
The detailed dated device evidence remains in the workspace's
`outputs/iDared-tested-app-compatibility-ledger.md` and its two narrative audits
and launch/app/error CSVs. They are historical evidence, not current pass lists.
The audit samples the full accessible history and relevant implementations; it
does not claim an exhaustive correctness review of every framework method.

## What the project actually is

The active fork is `bookieart1/iDared32bit`, branch
`ci/unsigned-ipa-baseline`. Upstream `touchHLE/touchHLE` was checked against
`ca1de89c578e79746f33c1332e28efd106c334ba`, also this checkout's merge base.
The local repository is shallow: counting an apparent upstream-to-HEAD log
without accounting for shallow boundaries can misattribute old work.

This remains high-level emulation, not a complete iPhone OS. Mach-O loading and
dyld stubs bridge guest ARM calls into Rust framework/libc implementations.
Objective-C dispatch shares a guest ABI bridge with host method implementations.
The guest has a flat 32-bit address space with strict null-page checks.
An Environment owns memory, CPU, framework state and cooperatively scheduled
guest threads represented by host coroutines. It moves through these coroutines;
diagnostics must follow the Environment, not a global or host-thread-local stack.

The inherited fork's principal architectural divergence from upstream is its
mGBA-derived ARM interpreter in place of Dynarmic JIT, extended for newer ARM,
Thumb, VFP and NEON instructions. UIKit/CoreAnimation, GLES 1 presentation and
OpenAL/audio adapters are other substantial areas of divergence. ES 2 is still
unsupported. The full fork diff includes inherited work predating this chat;
it cannot all be credited to our compatibility iterations.

## Historical evidence and correctness risks

| Area | Evidence and interpretation |
| --- | --- |
| Shared Foundation/runtime contracts | Earlier batches added collection, KVC, coding, date/string, URL and runtime APIs. The dated longitudinal audit found multiple apps advancing beyond prior blockers. Absence of a panic alone was never proof of working gameplay. Large batches prevent precise per-commit causal attribution. |
| Older persistence and VM work | The Spoing series crossed numeric/string archiving, calendars and POSIX/VM errors before confirmed play and persistence. Diversion advanced after missing-symbol handling and local ubiquitous key-value storage. These are aggregate observations, not proof that each commit independently caused an improvement. Local ubiquitous storage is not iCloud synchronization. |
| Memory fidelity | The flat memory model does not enforce per-page mprotect permissions. The added mprotect success response is therefore a limited accommodation, not a faithful protection contract. mmap/Mach VM bookkeeping and allocator state have separate overlapping-range handling; partial-range tests should verify that surviving ranges remain discoverable. The new recorder neither changes these behaviors nor treats readable memory as proof of live allocation. |
| UIKit and transitions | Menu/controller/animation/table changes across Dev 1.53–1.61 culminated in user-confirmed Spirits menus, gameplay, transitions and save/relaunch. The old tutorial-start crash is now an obsolete test premise. Table/slider/movie support remains incomplete outside that scope. |
| Networking/resource distinction | General NSURLRequest/NSURL implementations exposed Bounce's previously unreachable Burstly startup paths: a real regression in usable behavior. Dev 1.62 URL scheme support restored the earlier partial behavior. Dev 1.63 local-file NSURLConnection support was followed by confirmed Bulba gameplay and persistence. Offline remote connections and actual local resources must not be conflated. |
| Audio | Early AudioQueue success/silence placeholders did not establish playback. Subsequent queued decoding was materially different. Missing audio needs initialization, buffer consumption and output evidence, not merely successful return codes. |
| Rendering/input | Blot's tablet hierarchy/composition accommodation improved framing. It did not explain progressive Challenges→Back slowdown. Simultaneous audio/video degradation could be workload, scheduling or resource growth; it is not proof of a GPU defect. The ImageMagick single-thread workaround is still provisional. |
| Null/invalid objects | Splode's level-end null string/global-slot evidence and Addicus's PHInvocation/null dispatch evidence are different failures. Static references to a zeroed slot do not prove premature destruction. Hook games' nil dictionary value identifies a violated boundary, not the producer. They remain unresolved, with Hook games paused by user priority. |
| Existing accommodations | Legacy online-SDK fake classes, partial date formatting, approximate path/arc behavior and invalid-font fallbacks have limited contracts. They need explicit retirement/verification criteria; newly reached screens do not validate them globally. |
| Exception correctness | `NSException.raise` introduced in the 1.42 series logs and returns instead of unwinding. This is not correct exception handling and can hide the first contract violation. It is a priority for a separate strict-exception investigation, not silently changed in this build. The recorder retains raise entries as candidates. |

Latest raw Dev 1.65 log confirms B'Blast stops on an unsupported six-component
`sysctl` query, while Addicus still faults reading address zero at the
`PHInvocation` boundary. The previous change prevented a secondary diagnostic
panic; it did not fix Addicus gameplay. Spoing and Paper Monsters are useful
working controls. Bounce's return to its former behavior does not prove every
previous lag/audio problem resolved.

## Existing tools and the missing evidence

Already available: all-thread register/frame-pointer dumps; bounded final guest
code and caller context; Objective-C symbol lookup; targeted object/font lifetime
records; host-function identification at SVC; `--dump=linking-info`; GDB memory,
register, single-step, breakpoint and thread facilities; desktop GL apitrace.
Compile-time `log_dbg!` in ABI/dyld can trace calls, but requires rebuilding and
can produce enormous synchronous logs. These are useful tools, not replacements
for one another. Linking-info describes imports/classes, not the executed path.

The recurring missing fact is what happened *before* the last boundary: which
API produced nil, what query was requested, whether the caller returned, and
whether timers/presentation continued during slowdown. Final-instruction dumps
alone cannot answer that. Full instruction tracing or recording every memory
write would be expensive in this interpreter and is unnecessary for most
framework-contract failures.

## Prioritization

| Opportunity | Root-cause leverage / breadth | Cycle reduction | Risk / cost | Existing capability and validation |
| --- | --- | --- | --- | --- |
| Bounded call/return history plus progress | High for contract/order failures; broad | High: captures preceding evidence in one device run | Low semantic risk; bounded overhead must be measured | New missing link; test against B'Blast, Addicus, Blot and a working control |
| Reusable headless guest ABI fixtures | Very high for ABI/ownership correctness | Very high over time | Medium setup/maintenance | Old guest harness exists but is skipped on dev branch; repair separately |
| Offline binary/symbol workflow | High when a guest caller or global is known | High when the exact app binary is available locally | Low runtime risk, analyst effort | Existing linking-info/GDB/symbolizer underused; no matching game binaries available here |
| Strict exception audit | High correctness benefit; may reveal earlier causes | Potentially high | Changes behavior, can regress provisional compatibility | Known incorrect raise behavior; separate experiment with explicit rollback |
| Allocation/write provenance | High for selected stale pointers, not all nulls | Medium/high for targeted cases | Higher CPU/memory and interpreter integration cost | Some lifecycle diagnostics already exist; require a specific suspected object/slot |
| Full CPU or GLES/audio traces | High only after subsystem narrowed | Low if used indiscriminately | Highest volume, timing disturbance and interpretation cost | GDB/apitrace/debug modules already cover parts; add targeted probes later |

The first choice connects existing final-fault tools to the preceding history
without changing guest contracts. It also has a concrete no-device reproducer.
The next major process investment should be repairing the guest ABI fixture
pipeline, rather than indefinitely expanding device logging.

## Implemented boundary and interpretation

`--diagnostics=off|flight` defaults to off. This diagnostic build opts in only
B'Blast, Addicus, Blot and Spoing in DefaultOptions. These presets change capture,
not emulation; remove them after evaluation. User Options can override with
`--diagnostics=off`. No save data or package identity is changed.

Per enabled Environment, under 80 KiB of fixed storage retains:

- 96 recent API/Objective-C entry/return boundaries, ordered by sequence and
  paired by call ID, including guest thread, PC/LR/SP, raw register arguments,
  receiver/isa/class snapshot, and raw return registers. C calls include four
  readable stack words. History can wrap across callbacks or thread switches.
- 24 candidate events: known host object-returning methods returning nil,
  failed dlsym/open/fopen, and Objective-C raise entry. Nil can be legitimate;
  raw zero registers are never sufficient evidence of failure. Guest method
  return types are deliberately not guessed.
- Bounded argument capture for sysctl MIBs (up to eight words), symbol lookup
  names and resource paths (up to 80 bytes). Guest text is escaped and checked
  for readability. No descriptions or other guest calls are used to log objects.
- 24 five-second scheduler samples: cumulative API/ObjC calls, scheduler yields,
  main run-loop iterations, successful presentRenderbuffer returns, current
  thread PC/LR, runnable/allocated thread counts, and last observed main-loop
  timer/selector/audio queue/audio unit counts.

Recording performs no per-event formatting, output, heap allocation or clock
read. Routine ownership, GL and allocator/memory C calls are counted but omitted
from detailed history. A clock is read at existing scheduler boundaries.
At most 12 live progress lines are emitted per launch. A capped report is emitted
once on recoverable panic or return to the picker. Existing terminal register,
stack and memory diagnostics remain in place. No memory checks, callbacks,
return values, exception semantics or scheduling decisions are changed.

Interpretation: compare count deltas against elapsed milliseconds. Increasing
source counts can distinguish accumulating work from a fixed workload. A
presentation return is not proof that pixels are correct or even displayed.
`ObjC(host)` calls can inherit the last guest LR, so it is not an exact host
call site. Missing returns in a wrapped ring do not prove a blocked call.
Thread totals include retained dead slots; runnable counts are at sample time.

Limits: no guarantee of a final report after OS kill/native crash; no watchdog
while a host API blocks before yielding; no persistent full history; no guest
write provenance, decoding of arbitrary ObjC parameters, or instruction trace.
Unknown/missing selector text remains raw register evidence. Very frequent
normal nil results can overwrite older candidates. A slow game must remain open
long enough for progress samples, then return with the emulator Home button.

Files: `src/diagnostics.rs` owns the recorder; `environment.rs` connects API and
termination/scheduler boundaries; `objc/messages.rs` and `methods.rs` connect
dispatch and known return types; `ns_run_loop.rs` samples existing source counts;
options/help/defaults control capture. The existing CI gains one explicitly
visible synthetic-failure evidence step. There are no game compatibility fixes.

## Validation and device decision rule

Automated coverage includes ring wrapping/order, invalid diagnostic pointers,
interleaved thread identities, unchanged registers, real Objective-C empty-array
object versus numeric-zero results, progress/storage caps, and escaped text.
A synthetic six-component sysctl query passes through the actual dyld SVC
boundary with capture both off and on: the original unsupported-query panic
must remain and only the enabled run must preserve the MIB. Its values are
synthetic, not a claim to have recovered B'Blast's actual query.

Build/test results and artifact identity are recorded in the companion workspace
ledger after CI completes. The host has no native C++ build prerequisites;
local validation is source formatting/review. Hosted Windows/macOS library tests
and iOS compilation are distinct from guest ABI and physical-device tests.
The existing dev workflow skips full guest integration/native sanity/format
checks and permits lint failures; a green workflow is not a complete matrix.

Device test: launch B'Blast and Addicus once each; reproduce Blot's
Challenges→Back slowdown, leave it for about 20 seconds and use emulator Home;
play Spoing briefly and return Home as a working control. Send the ordinary log.
No settings edits or save resets are needed. Spirits is already confirmed
working, not a presumed crash reproducer. Sandwiches/SimplePhysics would add
new uncertain baselines and are deferred in this first evaluation.

Success means recovering an actual B'Blast MIB, a useful predecessor/return or
dispatch sequence for Addicus, and distinct progress trends around Blot's
slowdown, without destabilizing Spoing. If only the same final fault reappears,
do not automatically increase ring size: use the recovered caller/selector to
select a narrower probe or offline breakpoint. If timing becomes visibly worse,
reduce the selected event set before drawing compatibility conclusions.

## Durable workflow changes to keep

For each issue: retain exact binary/build identity, first decisive blocker,
competing causes, a discriminating observation, a minimal contract-level test,
and a focused device control. Separate implementation correctness, blocker
removal and observed gameplay/persistence. Keep diagnostic-only builds separate
from compatibility changes. Park low-value unresolved titles instead of adding
unverified accommodations; revisit when another title provides shared evidence.

Next priorities: restore a small strict guest ABI suite (struct returns,
varargs, callbacks and ownership); use exact game binaries with the existing
symbolizer/GDB when available; isolate strict exception behavior; add targeted
object/slot provenance only when the first recorder identifies a candidate.
Avoid adopting a modern API contract without checking historical iPhone OS ABI.

Primary references consulted:

- [Upstream debugging tools](https://github.com/touchHLE/touchHLE/blob/ca1de89c578e79746f33c1332e28efd106c334ba/dev-docs/debugging.md).
- [Upstream environment/scheduler](https://github.com/touchHLE/touchHLE/blob/ca1de89c578e79746f33c1332e28efd106c334ba/src/environment.rs).
- [QEMU tracing design](https://www.qemu.org/docs/master/devel/tracing.html):
  selective events and low-overhead capture, rather than indiscriminate printf.
  This implementation reuses this fork's boundaries; it does not import QEMU's
  tracing subsystem or claim equivalent capabilities.
