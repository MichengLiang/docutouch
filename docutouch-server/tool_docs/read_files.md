# `read_files` Deprecation Record

`read_files` existed as an experimental batch-read surface for LLMs. It wrapped
multiple file excerpts into one Diff-Inspired File Read Envelope (DIFRE) stream
so a caller could ask for several files in a single tool call.

Wave 0.5 removes that tool completely:

- the server no longer registers or dispatches `read_files`
- request parsing and server tests for `read_files` are removed
- the unused `docutouch-core` implementation is removed as well

## Why it existed

- it attempted to reduce tool-call count for multi-file context gathering
- it preserved 1-based absolute line coordinates inside a deterministic envelope
- it made the read stream explicitly non-executable, so it would not be
  confused with a real patch

## Why it was removed

- modern hosts can truncate or collapse oversized single-call payloads in ways
  that are hard to reason about after the fact
- when many files are concatenated into one giant return body, file boundaries,
  retry granularity, and citation granularity all get worse
- the tool had become a hidden-but-still-callable surface, which is worse than
  either fully supporting it or fully removing it
- ordinary `read_file` calls already cover the same job with better orchestration
  control and lower maintenance burden

## Preferred replacement

Use this pattern instead:

1. `list_directory` to establish the file set
2. repeated ordinary `read_file` calls at orchestration level
3. split large files with `line_range` when needed
4. summarize incrementally instead of aggregating many full file bodies into one
   monolithic tool response

This keeps file boundaries stable, makes retries precise, avoids giant payloads,
and better matches how current hosts handle tool output.


## read_files
```rust
build_tool::<ReadFilesArgs>(
                    "read_files",
                    indoc! {r#"
                        Batch extract target text file contents. The output stream strictly adheres to the Diff-Inspired File Read Envelope (DIFRE) protocol.
                
                        Identity:
                        - A deterministic batch file reading interface.
                        - Outputs raw file streams strictly mapped to absolute coordinates.
                        - Explicitly defined as a read-only scanning envelope; semantically rejects any association with executable unified patches.
                
                        Execution Semantics:
                        - Accepts batch requests inheriting the `relative_path`, `line_range`, and `show_line_numbers` schema from `read_file`.
                        - Fetches files sequentially and writes Blocks contiguously to the output stream without empty padding lines.
                
                        Global Addressing:
                        - All file-level coordinate computations are rigidly established as 1-based absolute line numbers.
                
                        Output Envelope Grammar (DIFRE):
                        ```text
                        Stream := { FileBlock }
                        FileBlock := Header Body
                        Header := "--- NUL" NEWLINE "+++ b/" path NEWLINE "@@ -0,0 +" startLine "," lineCount " @@" NEWLINE
                        Body := { Line }
                        ```
                
                        Output Rule Mechanics:
                        - `startLine` maps to the exact starting coordinate of the extracted block.
                        - `lineCount` denotes the total continuous integer count of lines within the block.
                        - When `show_line_numbers=true`, the `Line` structure strictly evaluates to: `<LineNumber> | <RawContent>`.
                        - When `show_line_numbers=false`, the `Line` structure evaluates to the unaltered `<RawContent>`.
                        - The `--- NUL` anchor constitutes an immutable starting signature, explicitly declaring the void of a left-side comparative state.
                    "#},
                )?,
```
