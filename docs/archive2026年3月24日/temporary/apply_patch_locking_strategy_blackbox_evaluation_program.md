# `apply_patch` Locking Strategy Black-box Evaluation Program

## Status

- Recorded on 2026-03-23.
- Scope: long-window black-box evaluation program for multiple locking strategies inside `apply_patch`.
- Authority level: temporary evaluation program, not a product spec.

## Purpose

The team has already chosen not to pre-commit to one permanent locking strategy
for `apply_patch`.

The purpose of this program is therefore:

- to compare ordinary context-matching authoring and line-locked authoring inside
  `apply_patch`
- under real host and model behavior
- over a long enough window that transient novelty effects do not dominate

## Evaluation Objects

### Strategy S1

- ordinary context-matching `apply_patch`

### Strategy S2

- line-locked authoring inside `apply_patch`

## Evaluation Principle

The program does not ask:

- which locking strategy appears more elegant in the abstract

It asks:

- which locking strategy produces better operational outcomes under realistic
  agent use

## Actors

### A1. Project owner

Consumes:

- periodic comparative findings
- default-strategy-readiness evidence

### A2. Primary implementation / orchestration agent

Consumes:

- error pattern summaries
- prompt teaching consequences
- escalation guidance improvements

### A3. Future collaborators and reviewers

Consume:

- durable evidence that a later default-strategy decision was based on observed
  behavior rather than stylistic preference

## Observation Dimensions

Each test session should record the following fields where applicable.

### Patch execution outcome

- full success rate
- partial success rate
- total failure rate

### Repair-loop cost

- retry count
- number of regenerated patches per task
- number of diagnostic turns before success

### Host interaction cost

- number of rereads after a write
- number of searches after a write
- total tool-call round trips

### Token-related cost

- patch request size
- observed follow-up read cost
- comparative total-turn cost where available

### Authoring quality

- grammar misuse patterns
- omission of required markers
- accidental ambiguity creation
- accidental fallback invocation frequency

### Human review quality

- human comprehensibility of authored patches
- diagnosability of failures
- surprise rate during review

### Cross-model stability

- ChatGPT/Codex-class behavior
- non-ChatGPT external model behavior
- family-specific misuse patterns

## Test Families

The comparison should include at least the following task families.

1. local single-chunk edits
2. repeated-pattern disambiguation
3. file-start insertion
4. same-file multi-chunk edits
5. multi-file coordinated refactors
6. edits under existing CRLF or no-final-newline conditions
7. long repetitive generated files
8. failure-and-repair loops after mismatch

## Controlled Variables

To keep comparisons meaningful, the following should be controlled where
possible:

1. same workspace state before each paired run
2. same task objective
3. same host class where possible
4. same baseline prompt policy except for strategy-specific tool guidance

## Logging Shape

Each observed task should produce one structured record with fields such as:

- task_id
- locking_strategy
- model_family
- task_family
- patch_success_status
- retries
- rereads_after_write
- round_trips
- notable_misuse_pattern
- reviewer_note

The program deliberately avoids overfitting to one metric.
The strategy choice should not be made on token count alone, nor on aesthetics
alone.

## Default-Strategy Decision Inputs

The eventual default-strategy decision should be based on a joined view across:

1. repair-loop efficiency
2. ambiguity resistance
3. cross-model robustness
4. human review readability
5. operational trust under real use

## Questions That Must Remain Open During Evaluation

The following questions must not be prematurely closed before sufficient black-
box evidence accumulates:

1. whether line-locked authoring should become the default recommended path
2. whether ordinary context-matching authoring should remain the normal first
   path for most edits
3. whether concise anchor form alone is enough in practice
4. whether fallback usage frequency is acceptable

## Deliverables

This program is expected to produce:

1. a durable observation log format
2. periodic comparative summaries
3. a future default-strategy judgment grounded in observed behavior

## Non-goals

This program does not itself:

- define the final grammar
- repair current defects
- declare a winning strategy in advance

Its role is evidence generation, not premature closure.
