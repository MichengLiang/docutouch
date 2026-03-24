# inkrail in DocuTouch

`script/inkrail.exe` is currently a small demo CLI for inspecting MyST label / ref relationships.

For this repository, it is now configured against `docs/source` through:

- `docs/source/.inkrail/config.toml`

The important project-specific fix is:

- `roots = ["."]`

Without that change, `inkrail` scans nothing because the generated default config pointed at a non-existent `docs` directory relative to `docs/source/.inkrail/`.

## What It Is Useful For Here

In the current DocuTouch corpus, `inkrail` is useful for four things:

1. Checking whether MyST labels are discoverable at all.
2. Locating the definition of a label quickly.
3. Finding incoming references to a label before renaming or restructuring a page.
4. Running lightweight broken-reference and duplicate-id checks.

It is not yet a full graph explorer for this corpus. In practice:

- `query --definition` works.
- `query --incoming` works.
- `check --broken` works.
- `check --duplicates` works.
- `stats` works.
- `query --outgoing`, `walk`, and `path` did not produce useful graph edges on the current corpus during validation.

Treat it as a backlink / lookup helper, not as authoritative graph traversal.

## Current Validation Snapshot

The current `docs/source` corpus was validated with:

```powershell
.\script\inkrail.exe stats .\docs\source
```

Observed result at the time of validation:

- `targets: 135`
- `refs: 78`
- `broken refs: 0`
- `duplicate ids: 0`

`check --orphans` reports many warnings on this corpus. That is expected and mainly means many labels do not currently have incoming refs.

## Commands To Use

Run commands from the repository root:

### 1. Show corpus summary

```powershell
.\script\inkrail.exe stats .\docs\source
```

Use this to confirm the tool is actually seeing the corpus.

### 2. Find where a label is defined

```powershell
.\script\inkrail.exe query docs-root-contract .\docs\source --definition
```

Good for jumping to the canonical location of a label.

### 3. Find who links to a page label

```powershell
.\script\inkrail.exe query docs-root-contract .\docs\source --incoming
```

This is the most useful command right now. Use it before:

- renaming a label
- moving a page
- merging pages
- deleting a page-level object

### 4. Check only broken refs

```powershell
.\script\inkrail.exe check .\docs\source --broken
```

For this project, this is the safest health check to run in day-to-day work.

### 5. Check duplicate ids

```powershell
.\script\inkrail.exe check .\docs\source --duplicates
```

This is useful after adding many new labels.

### 6. Inspect orphan labels explicitly

```powershell
.\script\inkrail.exe check .\docs\source --orphans
```

This is informational, not a hard failure signal for the current corpus.

## Practical Examples

### Example: inspect backlinks before editing a root page

```powershell
.\script\inkrail.exe query docs-root-contract .\docs\source --incoming
```

Validated result on this repository:

- `index.md:77`
- `30_records/20_migration/docs_markdown_migration_ledger.md:25`

### Example: inspect a stable interface page

```powershell
.\script\inkrail.exe query knowledge-interfaces-apply-patch-semantics .\docs\source --incoming
```

Validated result on this repository shows incoming refs from:

- `10_knowledge/60_decisions/apply_patch_warning_first_rationale.md`
- `10_knowledge/60_decisions/apply_splice_apply_patch_separation_rationale.md`
- `15_process_assets/10_exec_plans/apply_patch_semantics_hardening_plan.md`
- `index.md`

## Known Limits In This Repository

These behaviors were observed during validation:

- `query <target> --outgoing` returned no useful edges even when the page visibly contains `{ref}` links.
- `walk` returned empty outgoing expansions for page labels such as `docs-root` and `meta-index`.
- `path` could not find paths that should exist if page-level outgoing refs were modeled.

So for now, the reliable mental model is:

- labels are indexed
- incoming refs are indexed
- outgoing traversal is not yet reliable on this corpus

## Recommended Workflow

When editing `docs/source`, use `inkrail` like this:

1. Find the label you plan to change with `query --definition`.
2. Check backlinks with `query --incoming`.
3. Make the doc change.
4. Run `check --broken`.
5. If you introduced many new labels, also run `check --duplicates`.

This gives real value already, even with the current traversal limitation.
