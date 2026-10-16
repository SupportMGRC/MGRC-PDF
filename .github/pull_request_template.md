<!--
MGRC-PDF pull request. Work happens on `Noor`; pull requests go from `Noor` into `main`.
Delete any section that does not apply.
-->

## What this changes

<!-- What is different afterwards, and why. -->

## How it was checked

- [ ] `cargo fmt --all --check`
- [ ] clippy with `-D warnings` on the crates changed
- [ ] tests pass on the crates changed (a new bug-fix test was seen failing before the fix)
- [ ] `cargo xtask layers` and `cargo xtask assets` (if crate boundaries or assets changed)
- [ ] For UI changes: looked at the result (screenshot attached if useful)

## Never-crash (`AGENTS.md` §4)

- [ ] No new `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!` outside tests, unless provably
      infallible with a comment
- [ ] No input-derived indexing or slicing; checked arithmetic; capped allocations; bounded recursion
- [ ] No `unsafe`

## Assets and licences (`AGENTS.md` §1)

- [ ] No asset from an Adobe product; no font files added
- [ ] No GPL/AGPL code copied in
- [ ] Every new asset has an `ATTRIBUTION.toml` entry and `cargo xtask assets` passes
- [ ] MGRC logos and app icon used unmodified

## Security and privacy

- [ ] Still works offline; no new outbound network access
- [ ] MCP and the UI control channel stay opt-in, loopback-only and token-authenticated
- [ ] `--root` confinement still holds

## Docs

- [ ] `README.md`, `docs/` or the crate README updated if behaviour changed
