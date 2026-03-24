# Diagnostics Polish Closeout

## Status

- Recorded on 2026-03-23 as the Stage 6 closeout artifact for the current Rust
  DocuTouch diagnostics-polish wave.
- This is a bounded closeout record, not a new design-spec or roadmap.
- It is downstream of:
  - `docs/diagnostics_polish_spec.md`
  - `docs/temporary/diagnostics_polish_execution/diagnostics_polish_execution.md`
  - `docs/temporary/diagnostics_dx_repair_program.md`
- Superseded in part on 2026-03-23 by the accepted direction that keeps audit/report artifacts rejected but restores failed patch source persistence and full partial-failure enumeration as live contract work.

## 1. What Is Now Stable

The following points should now be treated as locked for this wave unless new
evidence appears:

1. Ordinary single full failure stays compact by default.
2. Partial success remains heavier because committed/failed/attempted accounting
   is genuinely repair-relevant.
3. Repeated equivalent `help:` guidance is not acceptable.
4. `caused by:` should add information rather than paraphrase the headline.
5. Empty patch belongs to the structured diagnostics family and is no longer an
   ad hoc plain-text branch.
6. High-value update-chunk mismatch failures may blame the first concrete
   expected old line when that is the truthful failure locus.
7. Per-unit repair guidance belongs to `failed_units`; top-level guidance should
   only exist when it is genuinely top-level.
8. Durable audit remains host-owned and must not be reintroduced at the tool
   layer.

## 2. What This Wave Achieved

This wave produced concrete contract gains across all four execution layers:

### 2.1 Design contract

- diagnostics polish now has a promoted live spec with an explicit judgment
  rubric and candidate register
- the local maximum for this subsystem has been defined
- “do not close” questions are now named rather than left implicit

### 2.2 Runtime behavior

- duplicate strategy guidance in the main failure path was removed
- single full failure no longer inherits partial-failure bulk by default
- empty patch now renders as a small structured outer-format failure
- a high-value mismatch subcase now points to the first concrete expected old
  line instead of only `@@`
- mirrored `help` ownership between top-level failure details and failed units
  has been narrowed

### 2.3 Automated protection

- regression tests now protect compact/full shape discipline, help dedupe, cause
  specificity, empty-patch unification, and several non-context failure families
- CLI/MCP parity now covers more high-value patch paths
- CLI contract automation now protects:
  - empty patch
  - no-op positional patch-file
  - `--patch-file` with spaced targets
  - move/write failure rollback behavior

### 2.4 Documentation trust

- live docs and public tool docs have been resynchronized with the current
  contract
- historical temporary docs are less likely to masquerade as live authority
- audit-heavy terminology has been substantially reduced or isolated

## 3. Residual Risk That Remains Real

The system is now in a high-quality band, but not at the local maximum.

The remaining real risks are:

1. **Source-span precision is still incomplete**
   - some execution failures still stop at coarse action/hunk anchors
   - target-side corroborating evidence is still sparse in some mismatch paths

2. **CLI black-box automation is stronger, but not complete**
   - the highest-value public patch cases are better covered
   - the full public CLI matrix is still not exhaustively automated

3. **Cross-layer ownership is safer, but not fully minimal**
   - the most obvious mirrored-help risk has been reduced
   - broader ownership/taxonomy simplification is still a possible future value
     area if it can be done without destabilizing DX

4. **Documentation trust can still drift over time**
   - historical material is safer than before
   - future work can still regress if temporary docs are allowed to regain
     authority by neglect

These are residual risks, not evidence that this wave failed.

## 4. Future Value That Is Still Legitimate

The following work remains legitimate future value if another bounded wave is
opened:

1. further source-span-grade execution diagnostics in the highest-value failure
   families
2. broader durable automation for public patch CLI contract edges
3. narrow additional ownership/taxonomy hardening where it directly reduces
   future DX regression risk
4. continued historical-doc trust cleanup if new misleading wording reappears

These remain valid because they still pass the promoted judgment rubric.

## 5. What Should Not Be Reopened

The following are now settled for this wave and should not be casually reopened:

1. whether single full failure should be compact
2. whether repeated equivalent `help:` lines are tolerable
3. whether the tool should keep its own durable audit layer
4. whether partial success should be flattened to look tidier
5. whether generic prose should replace specific low-level causes for style
6. whether empty patch should remain outside the diagnostics family

Reopening any of these without new evidence would be regression, not progress.

## 6. Why This Wave Is Bounded

This wave should now be considered bounded rather than recursively expandable
for three reasons:

1. The primary contract disputes have been resolved into live documentation.
2. The most costly DX regressions have been corrected in runtime behavior and
   protected by tests.
3. The remaining headroom is now concentrated finishing work, not broad
   conceptual ambiguity.

Continuing to recurse without a fresh bounded objective would risk:

- polishing churn
- taxonomy churn
- document sprawl
- review fatigue without proportional DX gain

That is exactly what the promoted rubric is meant to prevent.

## 7. Final Assessment

This diagnostics-polish wave should be treated as a successful bounded wave.

It did **not** make the subsystem perfect.
It **did** move the subsystem from “high-value DX repair still in flight” to
“high-quality contract with explicit residual risk.”

That is enough to justify closure of the current wave and transition to either:

- final human acceptance
- or a future separately bounded wave with a narrower target

## 8. Minimal Acceptance Packet

For final human acceptance, the main agent should be able to point to:

1. the promoted live spec:
   - `docs/diagnostics_polish_spec.md`
2. the active execution material:
   - `docs/temporary/diagnostics_polish_execution/diagnostics_polish_execution.md`
3. this closeout record:
   - `docs/temporary/diagnostics_polish_execution/diagnostics_polish_closeout.md`
4. passing regression suites in:
   - `codex-apply-patch`
   - `docutouch-core`
   - `docutouch-server`

That packet is sufficient to review this wave as a completed bounded effort.
