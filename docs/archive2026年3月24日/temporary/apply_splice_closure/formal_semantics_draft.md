# `apply_splice` Formal Grammar and Semantics Draft

## Status

- Purpose: implementation-driving draft for the `apply_splice` authored surface and execution semantics
- Scope: grammar, well-formedness, selection validation, omission rules, same-file semantics, byte fidelity, and program execution semantics
- Out of scope here: shared substrate extraction plan, module map, diagnostics taxonomy expansion, and test-plan decomposition
- The stable product boundary already lives in `docs/apply_splice_spec.md`; this
  draft narrows executable semantics and does not reopen tool identity

## 1. Scope and Contract Boundary

`apply_splice` is a structural-operation primitive over existing spans. It either transfers selected existing source bytes or removes a selected existing span, and it never authors new inline text. This matches the locked product boundary in `docs/apply_splice_spec.md` and the closure-stage summary (`docs/apply_splice_spec.md`, `docs/temporary/apply_splice_closure/stage_summary.md`).

The formal contract in this draft is therefore:

1. parse an authored splice program
2. validate each source and target selection against existing file snapshots
3. compose copy/move effects over those validated denotations
4. commit connected mutation units atomically
5. report `A/M/D`-style affected outcomes and failures in the existing DocuTouch style

This document intentionally does not define a free-form editing language.

## 2. Canonical Grammar

The canonical authored grammar is:

```ebnf
SpliceProgram      ::= BeginSplice LF SpliceAction+ EndSplice LF?

BeginSplice        ::= "*** Begin Splice"
EndSplice          ::= "*** End Splice"

SpliceAction       ::= TransferAction
                     | DeleteAction

TransferAction     ::= TransferSourceHeader LF SourceSelection LF TargetClause

DeleteAction       ::= DeleteHeader LF SourceSelection

TransferSourceHeader ::= "*** Copy From File: " Path
                       | "*** Move From File: " Path

DeleteHeader       ::= "*** Delete Span From File: " Path

TargetClause       ::= AppendTarget
                     | AnchoredTarget

AppendTarget       ::= LF "*** Append To File: " Path

AnchoredTarget     ::= LF TargetHeader LF TargetSelection

TargetHeader       ::= "*** Insert Before In File: " Path
                     | "*** Insert After In File: " Path
                     | "*** Replace In File: " Path

SourceSelection    ::= "@@" LF SourceSelectionBody
TargetSelection    ::= "@@" LF TargetSelectionBody

SourceSelectionBody ::= SelectionItem (LF SelectionItem)*
TargetSelectionBody ::= SelectionItem (LF SelectionItem)*

SelectionItem      ::= SelectionLine
                     | SourceOmission
                     | TargetOmission

SourceOmission     ::= "... source lines omitted ..."
TargetOmission     ::= "... target lines omitted ..."

SelectionLine      ::= PositiveInteger " | " VisibleContent

PositiveInteger    ::= "1" | NonZeroDigit Digit*
VisibleContent     ::= <the remaining bytes on the line, possibly empty>
```

Grammar-level notes:

- `Path` follows the same path rules already used by DocuTouch file tools: relative paths resolve against the active workspace or caller anchor; absolute paths remain absolute (`docutouch-server/src/server.rs:190-201`, `docutouch-server/src/server.rs:296-325`).
- The grammar admits empty `VisibleContent`. An empty visible line is written as `N | ` followed immediately by line break.
- `SelectionItem` is constrained further by authored-surface well-formedness rules below. In particular, a source selection may not contain `TargetOmission`, and a target selection may not contain `SourceOmission`.

## 3. Authored-Surface Well-Formedness Rules

After parsing, a splice program is well formed only if all of the following hold.

### 3.1 Action-shape rules

For every `SpliceAction`:

1. `*** Delete Span From File: ...` must not carry a target clause or target selection block.
2. `*** Append To File: ...` must not carry a target selection block.
3. `*** Insert Before In File: ...`, `*** Insert After In File: ...`, and `*** Replace In File: ...` must carry exactly one target selection block.
4. Each action has exactly one action header and exactly one source selection block.
5. No extra headers, comments, or alternative spellings are permitted.

These rules close the representation gap left intentionally open in `docs/temporary/apply_splice_implementation_plan.md:22-36`.

### 3.2 Selection-line rules

Within a single source or target selection body:

1. At least one `SelectionLine` must appear.
2. `SelectionLine` numbers must be strictly increasing.
3. Duplicate numbers are invalid.
4. Descending numbers are invalid.
5. Leading spaces in `VisibleContent` are significant.
6. Trailing spaces in `VisibleContent` are significant.
7. The syntax delimiter is exactly `digits + " | "`. The delimiter spaces are syntax, not content.
8. Horizontal truncation markers such as `...[N chars omitted]` are forbidden.

The last rule is mandatory because splice selections must denote exact existing text, not a shortened display form.

### 3.3 Omission-token rules

The authoritative omission tokens are:

- source selection: `... source lines omitted ...`
- target selection: `... target lines omitted ...`

Bare `...` is not canonical in `apply_splice`, even though sampled `read_file` may emit it for inspection (`docutouch-core/src/fs_tools.rs:545-551`, `docs/temporary/apply_splice_closure/stage_summary.md:13-19`).

Within one selection body:

1. An omission token must be surrounded by numbered lines on both sides.
2. An omission token may not be the first item.
3. An omission token may not be the last item.
4. Two omission tokens may not be adjacent.
5. An omission token between line `i` and line `j` requires `j > i + 1`.
6. A gap where `j > i + 1` without an omission token is invalid.
7. An omission token between adjacent numbered lines is invalid because it would denote an empty omission.

## 4. Formal File and Selection Model

For a file snapshot `F`, define its line decomposition as:

```text
Lines(F) = [L1, L2, ..., Ln]
Li = (body_i, sep_i, start_i, end_i)
```

where:

- `body_i` is the exact visible content bytes of logical line `i`
- `sep_i` is the exact separator bytes immediately following line `i`
- `sep_i ∈ { "\n", "\r\n", "" }`
- `start_i` is the starting byte offset of line `i`
- `end_i` is the first byte offset after `body_i || sep_i`

This model matches the existing line-oriented `read_file` rendering and separator splitting behavior, which already distinguishes `\r\n`, `\n`, and no separator (`docutouch-core/src/fs_tools.rs:497-507`, `docutouch-core/src/fs_tools.rs:572-579`).

### 4.1 Selection validation relation

Let `E` be a parsed source or target selection body consisting of numbered lines and omission tokens. Define:

```text
F ⊨ E ⇓ [a, b]
```

to mean that `E` validates against file snapshot `F` and denotes the contiguous line interval `[a, b]`.

`F ⊨ E ⇓ [a, b]` holds iff:

1. `E` contains at least one numbered line.
2. The first numbered line in `E` is `(a, text_a)` and the last numbered line is `(b, text_b)`.
3. `1 <= a <= b <= n`, where `n = |Lines(F)|`.
4. For every numbered line `(k, text_k)` appearing in `E`, `body_k = text_k`.
5. For every adjacent pair of numbered lines `(i, text_i)` then `(j, text_j)`:
   - if no omission token lies between them, then `j = i + 1`
   - if an omission token lies between them, then `j > i + 1`
6. `E` contains no forbidden omission placement and no horizontal truncation marker.

Consequences:

- the denoted interval is always contiguous
- omission is compression, not sparse sampling
- line numbers and visible content are both binding

This is the formal version of the design's double-lock rule (`docs/apply_splice_spec.md:172-190`).

### 4.2 Selection failure classes at the semantic boundary

This draft does not lock the final diagnostic code family, but the validation relation distinguishes at least these semantic failure classes:

- numbered line out of range
- numbered line content mismatch
- non-contiguous numbering without omission token
- omission token with illegal placement
- forbidden horizontal truncation marker
- wrong omission-token kind for the selection side

## 5. Target-Range Semantics

For target modes other than append, the target selection denotes a contiguous target range object `T = [t1, t2]` over the target file's original snapshot.

From `T`, derive two external boundary offsets:

- `pre(T) = start_t1`
- `post(T) = end_t2`

Then:

- `Insert Before In File` inserts at `pre(T)`
- `Insert After In File` inserts at `post(T)`
- `Replace In File` replaces byte span `[pre(T), post(T))`

Therefore a multi-line target selection is primarily a range object, not merely an anchor line. `Insert Before` and `Insert After` derive a boundary from that range; `Replace` consumes the full range.

For `Append To File`, no target range exists. The insertion point is the destination EOF boundary.

## 6. Byte-Span and Newline Semantics

For a validated source interval `[a, b]` over snapshot `F`, define the transferred source byte span as:

```text
Transfer(F, [a, b]) = Bytes(F)[start_a, end_b)
```

This includes:

- all visible bytes of lines `a..b`
- all separator bytes attached to lines `a..b`
- no synthetic separator beyond `end_b`

Consequences:

1. If line `b` ends with `\n`, that `\n` is transferred.
2. If line `b` ends with `\r\n`, that `\r\n` is transferred.
3. If line `b` is the final line of a file without a trailing newline, no newline is synthesized.
4. Mixed newline styles inside the source interval are preserved exactly.
5. The destination file's dominant newline style never rewrites transferred source bytes.

This directly instantiates the locked rule that source text is preserved verbatim, including newline bytes (`docs/apply_splice_spec.md:428-439`).

## 7. Same-File Semantics

### 7.1 Original-snapshot rule

For every commit unit and every touched file `P`, let `F0(P)` be that file's pre-mutation snapshot as observed before any action in the unit executes.

Formal rule:

- every source selection and every target selection that refers to `P` is validated against `F0(P)`
- no authored selection may refer to an intermediate state produced by another action in the same program

This strengthens the already-locked same-file rule in `docs/apply_splice_spec.md:387-400` into a program-level interpretation rule.

### 7.2 Rejection of intermediate-state-dependent interpretation

Intermediate-state-dependent interpretation is forbidden because it would make authored line numbers and visible-content locks order-dependent rather than observation-dependent.

Counterexample:

```text
Original file:
1 | alpha
2 | beta

Program:
*** Begin Splice
*** Copy From File: note.txt
@@
1 | alpha
*** Insert Before In File: note.txt
@@
2 | beta
*** Replace In File: note.txt
@@
2 | alpha
*** End Splice
```

If intermediate states were allowed, the second action could refer to the just-inserted `alpha`. Under the formal rule above, it cannot: line 2 in the original snapshot is `beta`, not `alpha`, so the second action is invalid. This rejection is intentional.

### 7.3 Same-file overlap rule

In v1, for same-file actions:

- if the source interval and target range overlap for `Insert Before`, `Insert After`, or `Replace`, the action is invalid
- the runtime must fail rather than infer intent

This is already a locked decision (`docs/apply_splice_spec.md:402-414`).

### 7.4 Same-file translation rule

For a same-file non-overlapping move or copy, the source and target denotations are computed first against `F0(P)`. Only after denotation may execution translate offsets.

Let source byte span be `[s1, s2)` and let the derived target insertion boundary or replacement span be based on original offsets:

- insertion boundary `b`
- replacement span `[r1, r2)`

For same-file `Move`:

- if inserting and `b <= s1`, translated boundary is `b`
- if inserting and `b >= s2`, translated boundary is `b - (s2 - s1)`
- if replacing and `r2 <= s1`, translated replacement span is `[r1, r2)`
- if replacing and `r1 >= s2`, translated replacement span is `[r1 - (s2 - s1), r2 - (s2 - s1))`

No third case exists in v1 because overlap is illegal.

## 8. Program-Level Execution Semantics

### 8.1 Commit-unit construction

`apply_splice` should follow the existing DocuTouch connected-unit model rather than whole-program global atomicity.

Given all actions in a splice program:

1. resolve each referenced source path and target path to a semantic path identity key
2. build an undirected graph where actions are connected if they touch a common path identity
3. each connected component becomes one commit unit

This aligns with the current patch runtime's path-identity grouping and connected commit-unit model (`codex-apply-patch/src/lib.rs:475-505`, `codex-apply-patch/src/lib.rs:667-729`).

### 8.2 Planning and commit phases

For each commit unit:

1. capture original snapshots `F0(P)` for all touched files
2. validate all selections against `F0`
3. compute all byte-span transfer and destination effects
4. reject any semantic conflicts before writes begin
5. derive final per-path post-state
6. commit that unit atomically

Commit behavior should inherit the current staged-write/rollback discipline: changed targets are backed up, new contents are staged, parent directories are created when writing, and failures roll back the whole unit (`codex-apply-patch/src/lib.rs:1216-1298`).

### 8.3 Missing-target policy

Target existence semantics are:

- `Append To File` may create a missing destination file
- when appending to a missing destination file, parent directories may be created during commit
- `Insert Before In File`, `Insert After In File`, and `Replace In File` require the target file and the target selection to already exist
- missing target for insert/replace is a semantic planning error before any write attempt

This follows the locked target-existence decision in `docs/apply_splice_spec.md:415-427` and the current atomic commit behavior that creates parent directories only while materializing writes (`codex-apply-patch/src/lib.rs:1258-1266`).

### 8.4 Outcome classification

Affected outcomes should follow the same coarse `A/M/D` style already used by DocuTouch patch tooling.

Representative cases:

- copy to missing destination file: `A dest`
- copy to existing destination file: `M dest`
- same-file copy: `M file`
- same-file move: `M file`
- cross-file move: `M dest`
- net-zero effect after composition: no affected entry

For cross-file move, the destination-side `M` baseline is already the implemented DocuTouch rule for move-shaped success summaries (`codex-apply-patch/src/lib.rs:940-955`).

### 8.5 Partial success

A splice program returns one of:

- full success: all commit units committed
- partial success: at least one unit committed and at least one unit failed
- failure: no unit committed

This follows the existing apply-patch status model and preserves the same agent-facing repair-accounting expectation (`codex-apply-patch/src/lib.rs:507-584`).

## 9. Minimal Examples

### 9.1 Copy + Append

```text
*** Begin Splice
*** Copy From File: source.py
@@
120 | def build_context(...):
121 |     return ctx
*** Append To File: target.py
*** End Splice
```

Semantics:

- validate source interval `[120, 121]`
- transfer exact bytes of those two lines, including their actual separators
- append at destination EOF
- source remains unchanged

### 9.2 Move + Replace

```text
*** Begin Splice
*** Move From File: source.py
@@
120 | def build_context(...):
... source lines omitted ...
128 |     return ctx
*** Replace In File: target.py
@@
45 | def old_context(...):
... target lines omitted ...
52 |     return old
*** End Splice
```

Semantics:

- validate contiguous source interval `[120, 128]`
- validate contiguous target interval `[45, 52]`
- replace the entire target byte span with the exact source byte span
- remove the source byte span
- if source and target are in one connected unit, commit atomically

### 9.3 Empty visible line

```text
*** Begin Splice
*** Copy From File: note.txt
@@
8 | first line
9 | 
10 | third line
*** Append To File: out.txt
*** End Splice
```

Line 9 denotes an empty visible line. Its separator bytes, if any, are preserved as part of the transferred span.

### 9.4 Invalid omitted gap

```text
@@
10 | alpha
12 | gamma
```

This is invalid because line 11 is skipped without an omission token.

### 9.5 Invalid trailing omission token

```text
@@
10 | alpha
... source lines omitted ...
```

This is invalid because omission tokens must be surrounded by numbered lines on both sides.
