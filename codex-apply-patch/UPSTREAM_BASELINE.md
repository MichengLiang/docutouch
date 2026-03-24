## Upstream Baseline

- Upstream repository: `https://github.com/openai/codex`
- Source subtree: `codex-rs/apply-patch`
- Baseline branch: `main`
- Baseline commit checked on 2026-03-18: `a265d6043edc8b41e42ae508291f4cfb9ed46805`

## Local Fork Policy

- Keep parser, invocation parsing, and seek/match behavior as close to upstream as possible.
- Record every intentional behavior divergence in `DOCUTOUCH_ENHANCEMENTS.md`.
- Treat this crate as a vendored fork with explicit sync notes, not as an opaque copy.

## Known Local Packaging Differences

- The crate is kept standalone instead of inheriting the upstream workspace manifest.
- Integration tests use Cargo's `CARGO_BIN_EXE_apply_patch` instead of upstream workspace helpers so the crate can be tested in isolation inside this repository.
