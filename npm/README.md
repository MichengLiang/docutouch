# docutouch

`docutouch` is a thin npm launcher for the `docutouch` binary.

It is not the primary product surface. The package downloads the matching GitHub Release binary for your platform on first run and then executes it.

## Supported Platforms

- Windows x64
- Linux x64

Other platforms fail with a clear error message.

## Install

Run without a global install:

```bash
npx docutouch --help
```

To start the stdio MCP server explicitly:

```bash
npx docutouch serve
```

Or install globally:

```bash
npm install -g docutouch
docutouch --help
```

## What It Downloads

The launcher resolves the current package version and downloads the matching raw binary asset from:

```text
https://github.com/MichengLiang/docutouch/releases/download/v${package.version}/...
```

Expected assets:

- `docutouch-x86_64-pc-windows-msvc.exe`
- `docutouch-x86_64-unknown-linux-gnu`

## Notes

- The first run needs network access so the launcher can fetch the binary.
- The downloaded binary is cached inside the installed package under `vendor/`.
- Environment variables such as `DOCUTOUCH_DEFAULT_WORKSPACE` and `DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE` still apply to the spawned binary.
