# Contributing

Thanks for helping. Bug reports, ideas and pull requests are all welcome.

## Before you start

- Search the [issues](https://github.com/DamienReichhart/Coffer/issues) first.
- For anything bigger than a small fix, open an issue to agree on the approach.
- Security problems: see [SECURITY.md](SECURITY.md), not public issues.

## Development

```bash
cargo run -p coffer
```

```bash
cargo test --workspace
```

Install 7-Zip to run the live tests (`SEVENZIP_REQUIRED=1` makes them mandatory, as in CI).

The website lives in `website/` (Node 22):

```bash
cd website && npm ci && npm run screenshots && npm run dev
```

## Rules of the road

- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` must be clean (clippy pedantic is on).
- New behaviour comes with tests. Command lines are unit-tested in `crates/core/src/command.rs`; parsing has fixtures from real 7-Zip output.
- Keep UI code out of `crates/core` and `crates/updater`.
- A change that alters a contract in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) updates every side of it in the same pull request.
- Add a line under **Unreleased** in `CHANGELOG.md` for anything users notice.
- Pull requests target `dev`; `main` is release-only (see [docs/RELEASING.md](docs/RELEASING.md)).

By contributing you agree that your work is released under the MIT licence.
