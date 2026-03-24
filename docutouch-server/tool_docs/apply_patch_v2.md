## `apply_patch`

Primary structural editing tool for applying concrete filesystem changes through patch-shaped input.

### Tool Identity

* Patch-shaped input only.
* The tool applies a concrete edit program to the filesystem.
* The tool is grounded in the OpenAI Codex `apply-patch` grammar and parser lineage, with a stronger file-group commit model in this runtime.
* The tool is not prose instruction input.
* The tool is not a preview mode.
* The tool is not a natural-language editing interface.

### Accepted Input Shape

* The accepted input is one literal patch text string.
* The outer patch envelope starts with `*** Begin Patch`.
* The outer patch envelope ends with `*** End Patch`.
* The patch body contains one or more file operations.
* Each file operation applies over a filesystem path and contributes to one patch execution.

### Minimal Grammar

```text
Patch := Begin { FileOp } End
Begin := "*** Begin Patch" NEWLINE
End := "*** End Patch" NEWLINE

FileOp := AddFile | DeleteFile | UpdateFile

AddFile := "*** Add File: " path NEWLINE { "+" line NEWLINE }  ; zero or more + lines
DeleteFile := "*** Delete File: " path NEWLINE
UpdateFile := "*** Update File: " path NEWLINE [ MoveTo ] { Hunk }

MoveTo := "*** Move to: " newPath NEWLINE

Hunk := "@@" [ header ] NEWLINE { HunkLine } [ "*** End of File" NEWLINE ]
HunkLine := (" " | "-" | "+") text NEWLINE
```

### Authoring Invariants

* Every file operation requires an explicit action header.
* `*** Add File: <path>` creates an add-shaped file operation. Every content line in its body must be a `+` line; zero content lines create an empty file.
* `*** Delete File: <path>` creates a delete-shaped file operation. Nothing follows inside that operation.
* `*** Update File: <path>` creates an update-shaped file operation. It may be followed immediately by `*** Move to: <new path>` and one or more hunks.
* If `*** Move to:` is used, place it immediately after `*** Update File:` and before the first hunk.
* Every hunk begins with `@@`, optionally followed by one explicit header.
* Every hunk body line must begin with exactly one of: space, `-`, or `+`.
* The patch body is patch syntax. Prose instructions are invalid input.

### Anchor Precision Escalation

* By default, prefer 3 lines of context above and 3 lines of context below each change.
* “3 lines” is a default authoring preference, not a required patch shape.
* If default context does not uniquely identify the target location, add an `@@` header such as `@@ class Example` or `@@ def handler():`.
* If one `@@` header is still insufficient, strengthen the patch with more local context or choose a more specific single `@@` header.
* Do not stack multiple `@@` header lines; the current parser consumes at most one explicit header per hunk.
* When adjacent changes fall within the same local region, do not duplicate overlapping context unless additional context is required for unique anchoring.

### Execution Semantics

* A single patch may contain multiple file operations.
* Repeated `*** Update File:` blocks for the same path are applied in patch order.
* `*** Delete File:` followed later by `*** Add File:` for the same path is allowed.
* `*** Move to:` applies after the updated file content is computed.
* Connected file operations commit atomically as one file group.
* Independent file groups may still succeed when another file group fails.
* Partial success is therefore possible across independent file groups, but never inside one connected file group.
* A net-zero patch may succeed with no affected files. In that case the runtime elides filesystem writes instead of touching file timestamps.

### Success Summary Semantics

* The common success block uses compact `A/M/D` outcome tags.
* These tags are a coarse affected-path summary.
* These tags are not a verb-by-verb replay of patch instructions.
* `A` reports an add-shaped outcome for a path.
* `M` reports a modify-shaped, update-shaped, or move-shaped outcome for a path.
* `D` reports a delete-shaped outcome for a path.
* Finer distinctions belong in warnings and failure diagnostics rather than in the common success path.

### Compatibility Notes

* `*** Add File:` is intended for creating a new file.
* In the current runtime, if the target already exists as a file, its contents are replaced.
* `*** Move to:` is intended for renaming to a destination path.
* In the current runtime, if the destination already exists as a file, its contents are replaced.

### Authoring Heuristics & Preferences

* These are soft recommendations to optimize execution and minimize conflict, not hard validity rules.
* Prefer `*** Update File:` when editing an existing file's content.
* Prefer a fresh destination path when renaming.

### Path Rules

* Relative paths resolve against the active workspace.
* Active workspace precedence is: explicit `set_workspace`, else valid `DOCUTOUCH_DEFAULT_WORKSPACE` loaded at server startup.
* If neither exists, relative paths fail and the runtime reports that a workspace must be set or that absolute paths must be used.
* Absolute paths are accepted.
* Path strings are interpreted as filesystem paths, not shell expressions.

### Failure Surface

* Outer-format errors fail before execution begins.
* Empty patch input is reported as a small structured outer-format failure rather than as an ad hoc plain-text branch.
* A failing file group leaves that file group unchanged.
* Independent file groups may already be committed when a later file group fails.
* Partial failure reports enumerate every committed and failed `A/M/D` path needed for safe repair; they do not compress committed or failed path accounting behind omission prose.
* Ordinary single full-failure diagnostics may render more compactly when an expanded failed-group block would only repeat already-visible repair facts.
* Execution-time failures point back to one primary patch-source location when that mapping is available robustly.
* Selected context-mismatch failures may also include one compact target-side anchor when the runtime has strong corroborating evidence.
* Failure output should keep wording compact, but not by hiding repair-critical committed or failed path information.
* When an inline patch fails and the runtime has a stable workspace anchor, diagnostics may reference a persisted failed patch source under a hidden workspace directory so the next repair step can reread the exact patch text.
* If no stable workspace anchor exists, failed patch source persistence may be unavailable; in that case the ordinary inline diagnostics remain the repair surface.
* DocuTouch does not emit extra audit or report artifacts as part of the repair contract.
* Broader audit trails still belong to the Codex host, which already retains tool-call receipts.

### Example

```text
*** Begin Patch
*** Add File: docs/todo.txt
+first item
*** Update File: src/app.py
@@ def greet():
-print("Hi")
+print("Hello")
*** Delete File: obsolete.txt
*** End Patch
```

### Operational Priorities

* Determine the intended file operation before drafting any body lines.
* Treat `Accepted Input Shape`, `Minimal Grammar`, and `Authoring Invariants` as validity boundaries.
* Treat `Anchor Precision Escalation` as the default path for improving location precision.
* Treat `Execution Semantics`, `Success Summary Semantics`, `Compatibility Notes`, `Path Rules`, and `Failure Surface` as interpretation boundaries for runtime behavior.
* Do not substitute generic diff habits for explicit runtime statements.

### High-Risk Misreadings

* Do not treat the tool as a channel for natural-language edit requests.
* Do not treat success summaries as complete execution logs.
* Do not treat a warning as a failure.
* Do not treat one local context-mismatch as evidence that the tool category is invalid.
* Do not project intended semantics onto the current runtime when both are explicitly separated.
* Do not treat an example as the definition space itself.

### Patch Construction Priorities

* Decide whether the change is add-shaped, delete-shaped, update-shaped, or move-shaped before writing local lines.
* Re-read the target content before authoring an update against a path that may have changed since the last observation.
* Keep each hunk anchored to the smallest region that still remains uniquely identifiable.
* Increase anchoring strength only when the current anchor is insufficiently unique.
* When `*** Move to:` is used, place it before the first hunk rather than after local change lines.
* Preserve the distinction between hard validity rules and soft authoring preferences while drafting.

### Hunk Discipline

* In an update hunk, body lines are patch lines, not freeform restatements of the file.
* Preserve the leading space, `-`, or `+` on every hunk body line. The prefix is part of the syntax, not decoration.
* Use unchanged context to anchor the change region, not to paraphrase surrounding content.
* When a header is used, treat it as an anchoring aid rather than as a substitute for a well-formed hunk body.
* A header line is not a hunk body line.
* Do not mechanically repeat a header line in the hunk body merely because that same text appears in the header.
* Do not introduce a second `@@` header line while trying to strengthen anchoring; use one explicit header plus surrounding context lines instead.
* When nearby edits share one local region, prefer one coherent anchored region over duplicated overlapping context.
* Include only the context actually needed to identify the change region and carry the edit.
* Do not invent a new hunk shape while trying to strengthen the anchor.

### Context Selection

* Start with the default local context.
* If that context is stale, non-unique, or too weak to identify the target region, strengthen the anchor.
* Use less than the default context when a smaller anchored region is already uniquely identifiable.
* Prefer the nearest stable lexical boundary that actually helps disambiguate the location.
* Prefer outer-to-inner anchoring only when one level is insufficient.
* Do not expand context merely for readability when uniqueness is already satisfied.

### Interpretation Discipline

* Read examples as anchors for joint recall, not as exhaustive templates.
* Read compatibility notes as scoped statements about divergence between abstract intent and current runtime behavior.
* Read path rules as runtime environment rules, not as grammar expansions.
* Read failure diagnostics as structured evidence about phase, group, and target, not as opaque prose.
* Read the inline diagnostics as the full repair object for ordinary failure handling.

### Failure Re-anchoring

* On a context-mismatch failure, first re-read the current target content.
* Then verify path, operation type, and patch shape before hypothesizing runtime defects.
* Then check whether the selected context is stale, non-unique, or over-expanded.
* Then strengthen anchoring with one or more `@@` header lines if uniqueness is still insufficient.
* Then use warnings and targeted diagnostics to refine the next patch.
* Do not collapse all failures into a single “tool broken” conclusion.

### Preference Heuristics

* Prefer `*** Update File:` when the path already exists and the goal is to change content in place.
* Prefer `*** Add File:` for new-path creation rather than replacement of existing content.
* Prefer `*** Move to:` when the semantic intent is rename or relocation after updated content is computed.
* Prefer a fresh destination path when renaming.
* Prefer one well-anchored hunk over multiple weakly anchored hunks in the same local region.

### Boundary-Preserving Judgments

* A valid patch shape does not by itself guarantee the intended runtime outcome.
* A runtime divergence does not retroactively change the grammar.
* A compact success summary does not authorize fine-grained replay inferences.
* A local failure does not invalidate already committed independent file groups.
* A sample patch demonstrates one admissible construction, not the full space of valid constructions.
