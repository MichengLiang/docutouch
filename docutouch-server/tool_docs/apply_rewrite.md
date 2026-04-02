# `apply_rewrite`

Primary structural rewrite tool for deleting or replacing numbered-selection-locked existing text spans, while also supporting file-level add, delete, and move operations.

## 1. Tool Identity

- Rewrite-program input only.
- The tool selects an existing contiguous old span, then deletes it or replaces it with authored new text.
- The tool is not a natural-language editing interface.
- The program body must be rewrite syntax, not prose instructions.

## 2. Minimal Grammar

```text
RewriteProgram := Begin { FileOp } End
Begin := "*** Begin Rewrite"
End := "*** End Rewrite"

FileOp := AddFile | DeleteFile | UpdateFile
AddFile := "*** Add File: " path WithBlock
DeleteFile := "*** Delete File: " path
UpdateFile := "*** Update File: " path [MoveTo] RewriteAction+
MoveTo := "*** Move to: " newPath

RewriteAction := Selection DeleteMarker | Selection WithBlock
Selection := "@@" [ " " intent_comment ] SelectionItem+
intent_comment := /(.*)/
SelectionItem := NumberedLine | Omission
NumberedLine := line_number " | " visible_text
Omission := "... lines omitted ..."
DeleteMarker := "*** Delete"

WithBlock := "*** With" TextLine* "*** End With"
TextLine := /(.*)/
```

## 3. Fast Decision Tree

Choose the rewrite shape by task type before writing any body text.

- To create a new file, use `*** Add File:` followed by `*** With`.
- To remove an existing file, use `*** Delete File:`.
- To edit an existing file, use `*** Update File:` followed by one or more rewrite actions.
- To rename while rewriting, use `*** Update File:` followed immediately by `*** Move to:`.
- To remove a selected old span, end that action with `*** Delete`.
- To replace a selected old span, end that action with `*** With ... *** End With`.
- To repair a failure, re-read the current target file and regenerate the numbered selection truthfully.

## 4. Canonical Rewrite Pattern

Every rewrite action starts with one selection block.
The selection identifies the old span.
The action then chooses exactly one ending:

- `*** Delete`
- `*** With ... *** End With`

### Replace one selected line

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ replace the selected print line with the production greeting
12 | print("Hi")
*** With
print("Hello")
*** End With
*** End Rewrite
```

### Replace a selected block

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ rewrite the greeting block to use a stable prefix
20 | def greet(name):
... lines omitted ...
22 |     return "Hi, " + name
*** With
def greet(name):
    prefix = "Hello"
    return prefix + ", " + name
*** End With
*** End Rewrite
```

### Delete a selected block

```text
*** Begin Rewrite
*** Update File: src/obsolete.py
@@ remove the obsolete helper after startup centralization
40 | def legacy_helper():
... lines omitted ...
47 |     return cached_value
*** Delete
*** End Rewrite
```

## 5. Selection Pattern

Selections are the only allowed old-side authoring surface.

### Bare `@@` header

```text
@@
88 | timeout = 30
```

### `@@` with one same-line intent comment

```text
@@ remove the legacy timeout override before loading config.mode
88 | timeout = 30
```

### Multi-line selection with omission

```text
@@ rewrite the existing context block without changing nearby code
120 | def build_context(...)
... lines omitted ...
128 |     return "strict"
```

### Dense selection when interior lines matter

```text
@@
55 | items = [
56 |     "alpha",
57 |     "beta",
58 | ]
```

Stable selection rules:

- Use absolute 1-indexed line numbers.
- Reproduce full visible line content for every numbered line you include.
- Use `... lines omitted ...` only for contiguous omitted interior lines.
- Do not use horizontal truncation.
- If you add a same-line `@@` comment, keep it short.
- The same-line `@@` comment is optional explanatory text only.
- The same-line `@@` comment never replaces the numbered selection body.
- Do not select a broad span merely to avoid writing another rewrite action.
- When the real edit consists of several distinct local changes, split them into separate rewrite actions so each selection stays meaningful and auditable.

### Good: split distinct local edits into separate actions

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ remove duplicate bootstrap helper
20 | def old_bootstrap(...):
... lines omitted ...
48 |     return bootstrap_state
*** Delete
@@ replace timeout assignment with config-backed value
70 | timeout = 30
*** With
timeout = config.timeout
*** End With
@@ delete obsolete compatibility branch
95 | if LEGACY_MODE:
... lines omitted ...
110 |     return settings
*** Delete
*** End Rewrite
```

### Not Good: one oversized action for several different edits

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ rewrite the startup section
20 | def old_bootstrap(...):
... lines omitted ...
110 |     return settings
*** With
[large rewritten block]
*** End With
*** End Rewrite
```

Not good. One oversized selection mixes several local edits, makes the `@@` comment vague, and increases unnecessary text churn.

## 6. `WithBlock` Pattern

`*** With` carries literal replacement text or full contents for `Add File`.

### Add a file

```text
*** Begin Rewrite
*** Add File: docs/rewrite-guide.md
*** With
# Rewrite Guide

Use numbered selections to rewrite existing spans.
*** End With
*** End Rewrite
```

### Replace a block

```text
*** Begin Rewrite
*** Update File: config/app.env
@@ rewrite the selected config block
10 | API_URL=http://localhost:3000
11 | DEBUG=true
*** With
API_URL=https://api.example.com
DEBUG=false
*** End With
*** End Rewrite
```

`WithBlock` rules:

- The payload is literal text.
- Do not prefix payload lines with `+`, `-`, or spaces.
- `*** End With` is mandatory.
- An empty `WithBlock` is allowed, but `*** Delete` is the preferred authored form for deletion.

Result-side line-boundary rules:

- If a replacement payload ends without a terminal newline and the rewritten result still keeps a suffix after that payload, the runtime preserves the line boundary during result composition instead of letting two lines collapse into one.
- If the replacement reaches file EOF and no suffix remains, the authored EOF state is preserved as written.

## 7. Add, Delete, and Move Patterns

### Delete a file

```text
*** Begin Rewrite
*** Delete File: obsolete.txt
*** End Rewrite
```

`*** Delete File:` targets an existing file only.
If the path is already missing, the operation hard-fails.

### Move and rewrite in one file operation

```text
*** Begin Rewrite
*** Update File: src/app.py
*** Move to: src/main.py
@@ rename the file and rewrite the app name constant
5 | APP_NAME = "demo"
*** With
APP_NAME = "production"
*** End With
*** End Rewrite
```

If a move-and-rewrite succeeds, the success summary uses final-path accounting:

```text
Success. Updated the following files:
A src/main.py
D src/app.py
```

### Multiple rewrite actions in one file

```text
*** Begin Rewrite
*** Update File: src/settings.py
@@ disable debug mode
10 | DEBUG = True
*** With
DEBUG = False
*** End With
@@ remove legacy mode
25 | LEGACY_MODE = True
*** Delete
*** End Rewrite
```

All rewrite actions in the same `Update File` are resolved against one original snapshot.
Do not author later selections against already-rewritten line numbers.

## 8. Multi-File Pattern

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ replace the greeting
12 | print("Hi")
*** With
print("Hello")
*** End With
*** Update File: src/config.py
@@ disable debug mode
4 | DEBUG = True
*** With
DEBUG = False
*** End With
*** Add File: docs/changelog.md
*** With
# Changelog

- Updated greeting and config defaults.
*** End With
*** Delete File: obsolete.txt
*** End Rewrite
```

Each file operation must be complete before the next file operation begins.

## 9. Common Invalid Shapes

### Comment-only `@@` header

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ remove the old block
*** Delete
*** End Rewrite
```

Invalid. The same-line `@@` comment is optional metadata only.
It cannot replace numbered selection lines.

### Multi-line `@@` comment

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ remove the old block
because startup changed
12 | old_value = 1
*** Delete
*** End Rewrite
```

Invalid. Any natural-language `@@` comment must stay on the same line as `@@`.
The next line must already be part of the numbered selection body.

### Patch-style hunk lines

```text
*** Begin Rewrite
*** Update File: src/app.py
@@
-print("Hi")
+print("Hello")
*** End Rewrite
```

Invalid. Rewrite actions use numbered selections plus `*** Delete` or `*** With`.

### Wrong omission marker

```text
@@
12 | start
... source lines omitted ...
19 | end
```

Invalid. Rewrite selections use only `... lines omitted ...`.

### Missing `*** End With`

```text
*** Begin Rewrite
*** Add File: notes.txt
*** With
hello
*** End Rewrite
```

Invalid. Every `WithBlock` must be explicitly closed.

### `*** Delete` and `*** With` in the same action

```text
*** Begin Rewrite
*** Update File: src/app.py
@@
10 | value = 1
*** Delete
*** With
value = 2
*** End With
*** End Rewrite
```

Invalid. One rewrite action must choose exactly one ending.

## 10. Repair Sequence After Failure

When a rewrite fails:

1. Re-read the current target file.
2. Verify the file operation type and target path.
3. Regenerate the selection with fresh absolute line numbers and exact visible text.
4. If multiple rewrite actions target the same file, check that their selected old spans do not overlap in the original snapshot.
5. If a `WithBlock` was malformed, rebuild the entire action with an explicit `*** End With`.
6. If a file-delete target is already gone, remove that delete from the rewrite program instead of retrying the same stale delete.
7. Regenerate only the failing file groups rather than rewriting unrelated successful groups.

## 11. Stable Preferences

- Prefer `apply_rewrite` when the real object is selection-locked old-span replacement or deletion.
- Prefer compact boundary-anchored selections for multi-line spans.
- Prefer `*** Delete` over teaching an empty `WithBlock` as the main deletion style.
- Prefer one truthful `Update File` with several non-overlapping rewrite actions over several loosely related file edits.
- Prefer `Add File` only for true creation and `Move to` only for true rename.

## 12. Summary

`apply_rewrite` works best when you model the task as selection first, rewrite second.

- Existing span replacement: selection plus `WithBlock`.
- Existing span deletion: selection plus `*** Delete`.
- New file: `Add File` plus `WithBlock`.
- File deletion: `Delete File`.
- Rename plus content change: `Update File` plus `Move to` plus rewrite actions.
- Multi-file change: write one complete file operation at a time.
- Failure: re-read the current file and regenerate the selection truthfully.

The preferred authoring style is narrow, explicit, and selection-led.
