(process-assets-apply-patch-line-number-assist-acceptance-criteria)=
# Apply Patch Line-Number-Assist Acceptance And QA Criteria

## Readiness Scope

Define the implementation-entry and release-entry readiness surface for `apply_patch` line-number-assisted locking, covering parser truthfulness, original-snapshot semantics, diagnostics, transport parity, and documentation truthfulness.

## Target Gate

- `apply-patch-line-number-assist-ready`

## Required Inputs

- stable decision source in {ref}`knowledge-decisions-apply-patch-numbered-anchor-guidance-rationale`
- candidate contract source in {ref}`deliberation-candidate-specs-apply-patch-line-number-assisted-locking-draft`
- current interface baseline in {ref}`knowledge-interfaces-apply-patch-semantics`
- parser, runtime, presentation, MCP, and CLI test evidence
- tool docs and examples that match the implemented behavior

## Open Risks

- parser may accept numbered evidence forms whose semantics are not explained truthfully in docs
- same-file multi-chunk numbering may silently drift if original-snapshot interpretation is not enforced consistently
- diagnostics may reveal line-number assist only partially, leaving repair behavior under-specified
- public docs may accidentally over-teach dense numbered evidence rather than the canonical single-anchor form

## Entry Conditions

- candidate spec is explicit enough to split implementation work without archive-only interpretation
- rollout plan and work package inventory already exist
- baseline product identity remains locked: one `apply_patch`, not a second patch tool

## Exit Conditions

- all top-level acceptance goals below are satisfied
- every criteria family has explicit pass evidence at the required layers
- permanent regression coverage exists for canonical success, mismatch, and parity paths
- stable docs, tool docs, tests, and examples describe the same observable contract

## Evidence Classes

| Evidence | Meaning |
| --- | --- |
| `E1` | decision/spec/readiness text matches the implemented behavior |
| `E2` | parser/runtime tests for numbered old-side evidence and snapshot semantics |
| `E3` | diagnostics/presentation evidence for visible success/failure behavior |
| `E4` | MCP/CLI transport parity evidence |
| `E5` | durable regression coverage in the permanent suite |

## Top-Level Acceptance Goals

- `AG-01 Single Public Canonical Form`: prompt-facing docs teach `@@ N | visible text` as the canonical auxiliary-location form.
- `AG-02 Truthful Parser Support`: parser support for numbered old-side evidence is explicit, bounded, and reflected in authored-blame locations.
- `AG-03 Original-Snapshot Semantics`: numbered old-side evidence is interpreted against the original snapshot of the `Update File` action, not against intermediate mutated state.
- `AG-04 Old-Side Double-Lock`: numbered evidence validates by absolute 1-indexed line number plus visible old-side content rather than by naked coordinate alone.
- `AG-05 Honest Diagnostics`: mismatch, malformed numbering, and unsupported authored shapes fail with truthful blame and repair-first wording.
- `AG-06 Transport And Doc Parity`: MCP, CLI, tool docs, and examples expose the same landed contract.
- `AG-07 Compatibility Truthfulness`: if unnumbered legacy authoring remains parseable, stable docs distinguish compatibility surface from preferred public guidance.

## Criteria Families

| Family | Core readiness question | Minimum evidence expectation |
| --- | --- | --- |
| `GRAM` | Is numbered evidence parsed exactly where intended and rejected where forbidden? | `E1`, `E2`, `E5` |
| `SNAP` | Are all numbered old-side semantics interpreted against the original snapshot? | `E1`, `E2`, `E5` |
| `LOCK` | Do line number and visible content operate as a double-lock rather than a loose hint? | `E1`, `E2`, `E3`, `E5` |
| `DIAG` | Do malformed numbering and mismatch failures point to truthful authored locations and repair guidance? | `E1`, `E3`, `E4`, `E5` |
| `PAR` | Do MCP and CLI present the same numbered-assist contract and visible outputs? | `E1`, `E4`, `E5` |
| `DOC` | Do stable docs, tool docs, and examples all teach the same canonical form without over-teaching wider parser support? | `E1`, `E5` |

## Minimum Gate Obligations

- Grammar gate: canonical `@@ N | visible text` parses; numbered old-side body evidence only parses in allowed old-side positions; `+` lines reject old-side numbering; malformed numeric prefixes fail deterministically.
- Snapshot gate: same-file multi-chunk patches do not reinterpret later numbered evidence against earlier mutations.
- Locking gate: successful numbered matching requires both the authored line number and visible text to agree with the original snapshot.
- Diagnostics gate: numbered mismatch, malformed numbering, and legacy/raw-text ambiguity all produce truthful blame and compact repair wording.
- Parity gate: CLI and MCP return the same success/failure semantics and no transport silently widens or narrows the contract.
- Documentation gate: stable docs and injected tool docs describe the same canonical authored shape before the feature is declared landed.

## Minimum Negative-Test Inventory

- malformed numbered anchor header
- non-positive or leading-zero line number
- missing delimiter after numbered prefix
- numbered prefix on `+` line
- line-number/content disagreement against target snapshot
- same-file multi-chunk case where intermediate-state interpretation would otherwise misapply a later chunk
- CLI/MCP parity mismatch on numbered-assist failures
- examples drifting away from the implemented canonical form

## Regression Requirements

- canonical single-anchor success path has durable parser/runtime/presentation coverage
- at least one dense numbered old-side evidence case is covered if parser support includes it
- every numbered mismatch family has both lower-layer and user-visible assertions
- transport-visible wording differences are covered explicitly, not by inference
- examples used in tool docs remain traceable to tests

## Definition Of Done

`apply_patch` line-number assist is ready for the target gate only when the canonical public form is stable, parser/runtime semantics are test-backed, original-snapshot interpretation is enforced, diagnostics tell the truth, CLI/MCP parity is green, and stable docs plus injected tool docs no longer overstate or understate the landed contract.
