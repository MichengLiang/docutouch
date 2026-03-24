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

```
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
* `*** Update File: <path>` creates an update-shaped file operation. It may be followed by `*** Move to: <new path>` and one or more hunks.
* Every hunk begins with `@@`.
* Every hunk body line must begin with exactly one of: space, `-`, or `+`.
* The patch body is patch syntax. Prose instructions are invalid input.

### Anchor Precision Escalation

* By default, prefer 3 lines of context above and 3 lines of context below each change.
* If default context does not uniquely identify the target location, add an `@@` header such as `@@ class Example` or `@@ def handler():`.
* If one `@@` header is still insufficient, strengthen the patch with more local context or choose a more specific single `@@` header.
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

```
*** Begin Patch
*** Add File: hello.txt
+Hello world
*** Update File: src/app.py
*** Move to: src/main.py
@@ def greet():
-print("Hi")
+print("Hello, world!")
*** Delete File: obsolete.txt
*** End Patch
```

```
*** Begin Patch
[ one or more file sections ]
*** End Patch
```

Within that envelope, you get a sequence of file operations.
You MUST include a header to specify the action you are taking.
Each operation starts with one of three headers:

*** Add File: <path> - create a new file. Every following line is a + line (the initial contents).
*** Delete File: <path> - remove an existing file. Nothing follows.
*** Update File: <path> - patch an existing file in place (optionally with a rename).

May be immediately followed by *** Move to: <new path> if you want to rename the file.
Then one or more “hunks”, each introduced by @@ (optionally followed by a hunk header).
Within a hunk each line starts with:

For instructions on [context_before] and [context_after]:
- By default, show 3 lines of code immediately above and 3 lines immediately below each change. If a change is within 3 lines of a previous change, do NOT duplicate the first change’s [context_after] lines in the second change’s [context_before] lines.
- If 3 lines of context is insufficient to uniquely identify the snippet of code within the file, use the @@ operator to indicate the class or function to which the snippet belongs. For instance, we might have:

```
@@ class BaseClass
[3 lines of pre-context]
- [old_code]
+ [new_code]
[3 lines of post-context]
```

- If a code block is repeated so many times in a class or function such that even a single `@@` statement and 3 lines of context cannot uniquely identify the snippet of code, strengthen the patch with more surrounding context lines or a more specific single `@@` header. Do not stack multiple `@@` header lines; the current parser consumes at most one explicit header per hunk. For instance:

```
@@ class BaseClass.def method
[3 lines of pre-context]
- [old_code]
+ [new_code]
[3 lines of post-context]
```
