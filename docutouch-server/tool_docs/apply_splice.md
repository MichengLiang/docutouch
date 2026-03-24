## `apply_splice`

Structural transfer tool for relocating, duplicating, or deleting already-existing text spans.

### Tool Identity

* Splice-program input only.
* The tool operates on existing text spans selected by numbered excerpts.
* The tool is not free-form authored text editing.
* The tool is distinct from `apply_patch`.

### Accepted Input Shape

* The accepted input is one literal splice text string.
* The outer envelope starts with `*** Begin Splice`.
* The outer envelope ends with `*** End Splice`.
* Each action uses one source clause and either one target clause or `Delete Span`.

### Current Supported Surface

```text
*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha
*** Append To File: dest.txt
*** End Splice
```

The current runtime supports the locked action basis:

* `Delete Span From File`
* `Copy From File` + `Append To File`
* `Move From File` + `Append To File`
* `Copy/Move From File` + `Insert Before In File`
* `Copy/Move From File` + `Insert After In File`
* `Copy/Move From File` + `Replace In File`

### Selection Contract

* Selections are line-oriented.
* Selections carry absolute 1-indexed line numbers.
* Omission tokens must be side-specific.
* Validation uses line numbers plus visible content.

### Path Rules

* Relative paths resolve against the active workspace or CLI current directory.
* Absolute paths remain legal without a workspace, using an execution-only anchor derived from the referenced paths.
* `Append To File` may create a missing destination file.
* Anchored target actions require the target file and target selection to already exist.

### Current Output Shape

* Success returns a compact `A/M/D` summary of affected files.
* Failures return compact `error[CODE]: ...` diagnostics with splice-specific codes.
* Partial success preserves committed `A/M/D` changes plus failed-unit repair accounting.

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

### Example

```text
*** Begin Splice
*** Move From File: source.txt
@@
2 | beta
*** Append To File: dest.txt
*** End Splice
```
