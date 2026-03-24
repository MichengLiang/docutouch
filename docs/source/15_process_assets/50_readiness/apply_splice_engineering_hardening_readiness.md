# (process-assets-apply-splice-engineering-hardening-readiness)=
# Apply Splice Engineering Hardening Readiness

## Readiness Scope

Define the execution-entry and merge-readiness gate for the `apply_splice` engineering hardening
wave, covering doctrine propagation, selection-authority cleanup, low-risk substrate extraction,
transport/presentation deduplication, and QA/doc closure.

## Target Gate

- `apply-splice-engineering-hardening-merge-ready`

## Required Inputs

- {ref}`process-assets-apply-splice-engineering-hardening-plan`
- {ref}`knowledge-architecture-apply-splice-architecture`
- {ref}`knowledge-operations-upstream-sync-and-compatibility`
- {ref}`knowledge-interfaces-apply-splice-spec`
- `30_records/60_status/apply_splice_implementation_status.md`
- concrete code-level findings on shared substrate seams, selection duplication, and
  transport/presentation drift risk

## Open Risks

- a refactor may accidentally push splice-owned semantics into a generic helper layer
- shared extraction may reduce duplication only cosmetically while keeping two real authorities
- transport deduplication may weaken splice-specific diagnostics or output wording
- docs and status pages may lag behind the changed architecture and recreate the same drift

## Entry Conditions

- accepted product-boundary doctrine is already stable
- current duplication seams are explicit enough to split into worker-owned slices
- the hardening wave is still scoped to maintainability/correctness, not a new product surface

## Exit Conditions

- each gate item below is closed or explicitly deferred with a bounded record sink
- changed seams have direct test evidence, not parity-only hope
- process assets and status/docs describe the post-wave architecture truthfully
- review can explain which seams were genuinely closed versus intentionally deferred

## Related Status Records

- future `30_records/60_status/` page for engineering hardening wave closeout

## Gate Table

| Gate Item | Current Standing | Blocking Risk | Required Action | Record Sink |
| --- | --- | --- | --- | --- |
| doctrine propagation into current wave hosts | open | workers execute against chat residue instead of canonical hosts | land plan/readiness hosts and keep them updated during the wave | future hardening-wave status page |
| splice selection-authority seam | open | text-level and byte-level selection logic drift apart | reduce or clearly re-anchor the duplicate implementation authority | future audit/status closeout |
| low-risk substrate extraction | open | affected-path/path-resolution logic keeps multiple owners | extract only genuinely semantic-free helpers or record bounded deferral | future audit/status closeout |
| transport/presentation shell duplication | open | source-path reporting and output plumbing drift tool-by-tool | reduce generic shell duplication while preserving splice-owned wording | future audit/status closeout |
| QA coverage for changed seams | open | refactor appears cleaner but is unproven | add regression evidence for each changed ownership seam | future audit/status closeout |
| docs/status truthfulness | open | architecture state is misdescribed after the wave | update docs/records to match the resulting ownership state | future hardening-wave status page |

## Minimum Gate Obligations

- Doctrine gate: workers can point to canonical process-asset hosts instead of relying on ad hoc
  chat instructions.
- Selection gate: the repo has one clearer authority for splice selection semantics or an explicit,
  reviewable split with bounded reasons.
- Substrate gate: any extracted helper is proven generic enough to serve both stacks without
  carrying splice-owned semantics.
- Transport gate: patch/splice shell reuse is improved where the logic is generic, and splice-owned
  wording remains explicit where the logic is not generic.
- QA gate: each changed seam has direct tests or focused assertions, not only transitive smoke
  coverage.
- Documentation gate: process assets, tool-facing docs, and status language no longer overstate
  or understate the actual architecture state.
