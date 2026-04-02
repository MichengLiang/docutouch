# `apply_patch`

Primary structural editing tool for applying concrete filesystem changes through patch-shaped input.

## 1. Tool Identity

- Patch-shaped input only.
- The tool applies a concrete edit program to the filesystem.
- The tool is not a shell command wrapper, not a preview mode, and not a natural-language editing interface.
- The patch body must be patch syntax, not prose.

## 2. Minimal Grammar

```text
Patch := Begin { FileOp } End
Begin := "*** Begin Patch"
End := "*** End Patch"

FileOp := AddFile | DeleteFile | UpdateFile
AddFile := "*** Add File: " path { "+" line }
DeleteFile := "*** Delete File: " path
UpdateFile := "*** Update File: " path [MoveTo] Hunk+
MoveTo := "*** Move to: " newPath

Hunk := "@@" [header] HunkLine+
HunkLine := (" " | "-" | "+") text
```

## 3. Fast Decision Tree

Choose the patch shape by task type before writing any local lines.

- To create a new file, use `*** Add File:`.
- To remove an existing file, use `*** Delete File:`.
- To modify an existing file, use `*** Update File:`.
- To rename an existing file, use `*** Update File:` followed immediately by `*** Move to:`.
- To strengthen location precision in a non-unique region, add one numbered `@@` header such as `@@ 120 | def handler():`.
- To repair a context mismatch, re-read the current target file and regenerate the patch with fresh local context.

## 4. Default Update Pattern

Use the smallest truthful patch that still identifies the target region clearly.

### Replace one line

```text
*** Begin Patch
*** Update File: src/app.py
@@
-print("Hi")
+print("Hello")
*** End Patch
```

### Replace a small contiguous block

```text
*** Begin Patch
*** Update File: src/app.py
@@
 def greet(name):
-    return "Hi, " + name
+    return "Hello, " + name
*** End Patch
```

### Replace a block with surrounding context

```text
*** Begin Patch
*** Update File: src/app.py
@@
 def greet(name):
-    prefix = "Hi"
-    return prefix + ", " + name
+    prefix = "Hello"
+    return prefix + ", " + name
 
 def main():
*** End Patch
```

## 5. Numbered Anchor Pattern

Use one numbered `@@` header when ordinary local context is not specific enough.

### Repeated function body

```text
*** Begin Patch
*** Update File: src/app.py
@@ 120 | def handler():
-    value = 1
+    value = 2
*** End Patch
```

### Repeated list region

```text
*** Begin Patch
*** Update File: docs/guide.md
@@ 88 | 1. Install dependencies
-2. Run tests
+2. Run the full test suite
*** End Patch
```

### Numbered header followed by repeated first old-side line

```text
*** Begin Patch
*** Update File: docs/list.md
@@ 101 | 1. first item
-1. first item
-2. second item
+1. first item updated
+2. second item updated
*** End Patch
```

The public guidance remains one numbered header plus ordinary patch lines. The repeated-first-old-side shape is accepted as a narrow compatibility form when the first old-side line repeats the same visible text as the numbered header.

## 6. Add, Delete, and Move Patterns

### Create a file

```text
*** Begin Patch
*** Add File: docs/todo.txt
+first item
+second item
*** End Patch
```

### Delete a file

```text
*** Begin Patch
*** Delete File: obsolete.txt
*** End Patch
```

### Rename and update content

```text
*** Begin Patch
*** Update File: src/app.py
*** Move to: src/main.py
@@
-print("Hi")
+print("Hello")
*** End Patch
```

## 7. End-of-File Patterns

### Append near the file end

```text
*** Begin Patch
*** Update File: src/app.py
@@
 existing_line
+new_last_line
*** End Patch
```

### Replace the final line

```text
*** Begin Patch
*** Update File: src/app.py
@@
-old last line
+new last line
*** End Patch
```

Use ordinary patch lines. Do not invent a custom EOF syntax unless the patch shape explicitly requires it.

## 8. Multi-File Pattern

```text
*** Begin Patch
*** Update File: src/app.py
@@
-print("Hi")
+print("Hello")
*** Update File: src/config.py
@@
-DEBUG = True
+DEBUG = False
*** Add File: docs/changelog.txt
+Updated greeting and config defaults.
*** End Patch
```

Each file operation must be complete before the next file operation begins.

## 9. Common Invalid Shapes

### Prose inside the patch body

```text
*** Begin Patch
Please update app.py to print hello
*** End Patch
```

Invalid. The body must be patch syntax.

### Body lines without prefixes

```text
*** Begin Patch
*** Update File: src/app.py
@@
print("Hi")
print("Hello")
*** End Patch
```

Invalid. Each hunk body line must begin with space, `-`, or `+`.

### Multiple stacked `@@` headers

```text
@@ class App
@@ def handler():
-...
+...
```

Invalid for the current parser contract. Use one explicit header plus local context lines.

## 10. Repair Sequence After Failure

When a patch fails with a context mismatch:

1. Re-read the current target file.
2. Verify the target path and file operation type.
3. Replace stale context with fresh local context.
4. Add one numbered `@@` header if the target region is still not unique enough.
5. Regenerate only the failing file groups rather than rewriting unrelated successful groups.

## 11. Stable Preferences

- Prefer `*** Update File:` when editing an existing file in place.
- Prefer `*** Add File:` only for true creation.
- Prefer a fresh destination path when renaming.
- Prefer one well-anchored hunk over several weakly anchored hunks in the same local region.
- Prefer fresh context over invented syntax.

## 12. Summary

`apply_patch` works best when the patch shape matches the task shape.

- Small edit: use a small truthful hunk.
- Non-unique target: add one numbered header.
- New file: use `Add File`.
- Deleted file: use `Delete File`.
- Rename: use `Move to` immediately after `Update File`.
- Multi-file change: write one complete file operation at a time.
- Failure: re-read the target and regenerate with fresh context.

The preferred authoring style is narrow, explicit, and example-driven: choose the task category first, then follow the matching patch pattern.
