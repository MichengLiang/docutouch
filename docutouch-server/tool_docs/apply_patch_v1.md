Apply a freeform patch. This is the primary structural editing tool.

Identity:

- Patch-shaped input only. Prose instructions are invalid.
- The tool applies a concrete edit program to the filesystem.
- The tool is grounded in the OpenAI Codex `apply-patch` grammar and parser lineage, with a stronger file-group commit model in this runtime.
- The tool is not a shell command wrapper, not a preview mode, and not a natural-language editing interface.

Accepted Input Shape:

- One literal patch text string.
- The patch envelope starts with `*** Begin Patch`.
- The patch envelope ends with `*** End Patch`.
- The patch body contains one or more file operations.

Minimal Grammar:

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

Patch Writing Guidance:

- New files use `*** Add File: <path>` and every body line is a `+` line; zero body lines create an empty file.
- Existing files use `*** Update File: <path>` and one or more hunks.
- Deleted files use `*** Delete File: <path>` and nothing follows.
- Renames use `*** Move to: <new path>` immediately after `*** Update File: ...`.
- By default, prefer 3 lines of context above and below each change.
- If 3 lines of context are not unique enough, add one numbered `@@` anchor such as `@@ 120 | def handler():`.
- The default public guidance teaches one numbered anchor, not raw textual `@@ class Example` / `@@ def handler():` headers.
- If one numbered anchor is still not specific enough, strengthen the patch with fresher surrounding context or a more local truthful anchor. Do not stack multiple `@@` header lines; the current parser consumes at most one explicit header per hunk.

Execution Semantics:

- Multiple file operations may appear in one patch.
- Repeated `*** Update File:` blocks for the same path are applied in patch order.
- `*** Delete File:` followed later by `*** Add File:` for the same path is allowed.
- `*** Move to:` applies after the updated file content is computed.
- Connected file operations commit atomically as one file group.
- Independent file groups may still succeed when another file group fails.
- Partial success is therefore possible across independent file groups, but never inside one connected file group.

Success Summary Semantics:

- The common success block intentionally uses compact `A/M/D` outcome tags.
- These tags are a coarse affected-path summary, not a verb-by-verb replay of patch instructions.
- `A` reports an add-shaped outcome for a path, `M` reports a modify/update/move-shaped outcome, and `D` reports a delete-shaped outcome.
- `Update File` and `Move to` commonly surface as `M` in the success summary.
- Finer distinctions belong in warnings and failure diagnostics rather than the common success path.

Compatibility Notes:

- `Add File` is intended for creating a new file.
- In the current runtime, if the target already exists as a file, its contents are replaced.
- `Move to` is intended for renaming to a destination path.
- In the current runtime, if the destination already exists as a file, its contents are replaced.
- Prefer `Update File` when editing an existing file.
- Prefer a fresh destination path when renaming.

Path Rules:

- Relative paths resolve against the active workspace.
- Active workspace precedence is: explicit `set_workspace`, else valid `DOCUTOUCH_DEFAULT_WORKSPACE` loaded at server startup.
- If neither exists, relative paths fail and tell the caller to set workspace or use absolute paths.
- Absolute paths are accepted.
- Path strings are interpreted as filesystem paths, not shell expressions.

Failure Surface:

- Outer-format errors fail before execution begins.
- Empty patch input is reported as a small structured outer-format failure rather than as an ad hoc plain-text branch.
- A failing file group leaves that file group unchanged.
- Independent file groups may already be committed when a later file group fails.
- Partial failure reports enumerate every committed and failed `A/M/D` path needed for safe repair; they do not compress committed or failed path accounting behind omission prose.
- Ordinary single full-failure diagnostics may render more compactly when an expanded failed-group block would only repeat already-visible repair facts.
- Execution-time failures point back to one primary patch-source location when that mapping is available robustly.
- Selected context-mismatch failures may also include one compact target-side anchor when the runtime has strong corroborating evidence.
- Failure output should keep wording compact, but not by hiding repair-critical committed or failed path information.
- When an inline patch fails and the runtime has a stable workspace anchor, diagnostics may reference a persisted failed patch source under a hidden workspace directory so the next repair step can reread the exact patch text.
- If no stable workspace anchor exists, failed patch source persistence may be unavailable; in that case the ordinary inline diagnostics remain the repair surface.
- DocuTouch does not emit extra audit or report artifacts as part of the repair contract.
- Broader audit trails still belong to the Codex host, which already retains tool-call receipts.
- A net-zero patch may succeed with no affected files; in that case the runtime elides filesystem writes instead of touching file timestamps.

Example:

```text
*** Begin Patch
*** Add File: docs/todo.txt
+first item
*** Update File: src/app.py
@@ 12 | def greet():
-print("Hi")
+print("Hello")
*** Delete File: obsolete.txt
*** End Patch
```
