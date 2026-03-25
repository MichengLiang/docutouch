## `apply_splice`

Primary structural transfer tool for relocating, duplicating, or deleting already-existing text spans.

### Tool Identity

* Splice-program input only.
* The tool operates on already-existing text spans selected by numbered excerpts.
* The tool expresses source-to-target transfer relations or source-only deletion.
* The tool does not author new inline text inside the splice program.
* The tool is distinct from `apply_patch`.

### Accepted Input Shape

* The accepted input is one literal splice text string.
* The outer envelope starts with `*** Begin Splice`.
* The outer envelope ends with `*** End Splice`.
* The body contains one or more splice actions.
* Each action is either:
  * one source clause plus one target clause, or
  * one `Delete Span` source clause with no target clause.

### Minimal Grammar

```text
Splice := Begin { Action } End
Begin := "*** Begin Splice" NEWLINE
End := "*** End Splice" NEWLINE

Action := DeleteAction | TransferAction

DeleteAction := DeleteSource Selection
TransferAction := TransferSource Selection Target

DeleteSource := "*** Delete Span From File: " path NEWLINE
TransferSource := ("*** Copy From File: " | "*** Move From File: ") path NEWLINE

Target := AppendTarget | InsertBeforeTarget | InsertAfterTarget | ReplaceTarget
AppendTarget := "*** Append To File: " path NEWLINE
InsertBeforeTarget := "*** Insert Before In File: " path NEWLINE Selection
InsertAfterTarget := "*** Insert After In File: " path NEWLINE Selection
ReplaceTarget := "*** Replace In File: " path NEWLINE Selection

Selection := "@@" NEWLINE { SelectionItem NEWLINE }
SelectionItem := NumberedLine | SourceOmission | TargetOmission
NumberedLine := line_number " | " visible_content
SourceOmission := "... source lines omitted ..."
TargetOmission := "... target lines omitted ..."
```

### Authoring Invariants

* Every splice program must contain at least one action.
* Every transfer action requires exactly one source clause, one source selection, and one target clause.
* `Delete Span` actions do not admit a target clause.
* Every selection must begin with `@@`.
* Every selection must contain at least one numbered line.
* Numbered lines must use the exact `N | content` delimiter.
* Numbered line numbers must be positive integers, must not use leading zeroes, and must be strictly increasing.
* Non-contiguous numbered lines require an omission token.
* Omission tokens are side-specific and must be surrounded by numbered lines.
* Consecutive omission tokens are invalid.
* Selection lines must reproduce full visible line content, not sampled inspection fragments.

### Action Basis

The current runtime supports the locked action basis:

* `Delete Span From File`
* `Copy From File` + `Append To File`
* `Move From File` + `Append To File`
* `Copy From File` + `Insert Before In File`
* `Copy From File` + `Insert After In File`
* `Move From File` + `Insert Before In File`
* `Move From File` + `Insert After In File`
* `Copy From File` + `Replace In File`
* `Move From File` + `Replace In File`

### Action Authoring Preference

* Prefer `Delete Span From File` when removing an existing contiguous span without target-side transfer.
* For larger removals, prefer `Delete Span` over restating the removed body through a patch-style deletion.

### Canonical Authored Shapes

Append shape:

```text
*** Begin Splice
*** Copy From File: source.py
@@
12 | def build_context(...)
... source lines omitted ...
19 |     return "strict"
*** Append To File: target.py
*** End Splice
```

Replace shape with source-side and target-side omission:

```text
*** Begin Splice
*** Move From File: source.py
@@
120 | def build_context(...)
... source lines omitted ...
128 |     return "strict"
*** Replace In File: target.py
@@
45 | old block start
... target lines omitted ...
52 | old block end
*** End Splice
```

Delete shape:

```text
*** Begin Splice
*** Delete Span From File: source.py
@@
40 | def obsolete_helper(...)
... source lines omitted ...
47 |     return old_value
*** End Splice
```

### Selection Contract

* Selections are line-oriented numbered excerpts.
* Selections carry absolute 1-indexed line numbers.
* Validation uses double-lock matching: line numbers plus visible line content.
* Omission tokens are authoritative, side-specific range compression markers.
* Omission tokens denote contiguous omitted lines, not sparse sampling.
* Source selections use `... source lines omitted ...`.
* Target selections use `... target lines omitted ...`.

### Selection Authoring Preference

* Prefer omission-backed boundary anchors for multi-line selections.
* Default to one starting numbered line, one omission token, and one ending numbered line when the omitted interior is contiguous and not independently required.
* Expand interior numbered lines only when the shorter boundary-anchored form would be ambiguous or when the full interior span is itself the intended evidence.
* For anchored target actions, prefer the shortest truthful target selection that still locks the intended anchor or replacement range.
* For every numbered line you do include, reproduce the full visible line content.

### Same-File And Destination Rules

* Same-file source and target selections are resolved against one original snapshot.
* Same-file anchored target actions are allowed when the source and target ranges remain non-overlapping against that original snapshot.
* Same-file overlap is illegal only when the selected source range overlaps the anchored target range in that original snapshot.
* `Append To File` may create a missing destination file.
* `Insert Before In File`, `Insert After In File`, and `Replace In File` require the target file and target selection to already exist.
* Ordinary transfer keeps the selected source bytes and newline bytes intact where that does not break line structure in the target result.
* When the selected source range ends on the source file's final line without a terminal newline, and the target-side boundary would otherwise concatenate two lines, runtime normalizes the transfer result with a target-style line separator.
* This newline-boundary normalization applies only to the target/result composition for the current action; it does not rewrite the source file's own EOF state.

### Execution Semantics

* `Copy + Append` appends the selected source bytes to the destination and leaves the source unchanged.
* `Move + Append` appends the selected source bytes to the destination and removes the selected source bytes.
* `Copy/Move + Insert Before/After` uses the validated target selection as the insertion anchor.
* `Copy/Move + Replace` replaces the validated target range with the selected source bytes.
* `Delete Span` removes the validated source range and performs no target write.
* Transfer-family actions must preserve line separation in the target result; they do not intentionally create same-line concatenation from an EOF-without-newline source selection.
* Each splice action is a connected mutation unit.
* Disjoint connected units may partially succeed.
* A connected mutation unit must not leave a move half-applied.

### Success Summary Semantics

* Success returns a compact `A/M/D` affected-path summary.
* These tags summarize affected paths, not source/target verbs.
* `A` reports an add-shaped path outcome.
* `M` reports a modify-shaped path outcome.
* `D` reports a delete-shaped path outcome.

### Path Rules

* Relative paths resolve against the active workspace or CLI current directory.
* Absolute paths remain legal without a workspace, using an execution-only anchor derived from the referenced paths.
* Path strings are interpreted as filesystem paths, not shell expressions.

### Failure Surface

* Failures return compact `error[CODE]: ...` diagnostics with splice-specific codes.
* Source mismatch blames the source selection.
* Target mismatch blames the target selection.
* Same-file legality failures blame the action header.
* Write-stage failures preserve repair-oriented accounting.
* Partial success preserves committed `A/M/D` changes plus failed-unit accounting.

### Current Diagnostic Family

* `SPLICE_PROGRAM_INVALID`
* `SPLICE_SOURCE_SELECTION_INVALID`
* `SPLICE_TARGET_SELECTION_INVALID`
* `SPLICE_SELECTION_TRUNCATED`
* `SPLICE_SOURCE_STATE_INVALID`
* `SPLICE_OVERLAP_ILLEGAL`
* `SPLICE_TARGET_STATE_INVALID`
* `SPLICE_WRITE_ERROR`
* `SPLICE_PARTIAL_UNIT_FAILURE`
