## DocuTouch Enhancements

This fork keeps the official `apply-patch` grammar and content-matching behavior as the semantic baseline, but deliberately strengthens the filesystem commit model for LLM editing workflows.

For the current design record on compatibility disclosure, triggered warnings, path identity, and Windows hardening priorities, see `../docs/apply_patch_semantics_plan.md`.

### 1. File-group commit model

Upstream behavior applies hunks sequentially and may leave earlier filesystem changes behind when a later hunk fails.

This fork now:

- groups hunks by connected file paths
- treats each connected group as one commit unit
- guarantees all-or-nothing commit inside each unit
- still allows independent units to succeed when another unit fails

This is the main functional divergence from upstream.

### 2. Structured execution report

The fork adds a structured runtime report that distinguishes:

- `FullSuccess`
- `PartialSuccess`
- `Failure`

This keeps the CLI behavior simple while allowing higher layers such as DocuTouch MCP bindings to expose partial-apply outcomes precisely.

Failed commit units now also carry structured diagnostics instead of only a flat message:

- stable failure codes
- target path metadata
- action / hunk indices when available
- primary patch-source line / column when that mapping is available
- intended `A/M/D` changes for the failed unit
- per-failure help text for repair-oriented UX

This lets the server render partial success with both committed changes and failed groups in a consistent `A/M/D`-aware shape.

The same metadata feeds the MCP server's inline diagnostics and partial-failure presentation without requiring a second tool-managed artifact layer.

### 3. Atomic multi-file execution

Commit units are now applied through staged sibling backups and staged writes so that multi-path operations such as moves either fully land or fully roll back inside the unit.

### 4. Standalone testability

The local fork removes the upstream workspace-only test launcher dependency and uses standard Cargo integration-test binary discovery so the crate can be validated inside this repository without the full upstream workspace.
