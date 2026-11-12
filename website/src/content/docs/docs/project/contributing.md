---
title: Contributing
description: How to report problems, suggest features and contribute code or docs.
---

Contributions are welcome — bug reports, ideas, documentation fixes and code.

## Reporting a bug

Open an [issue](https://github.com/DamienReichhart/7zip-GUI/issues/new/choose) with:

- your operating system and architecture, the 7zip-GUI version (**About**) and the 7-Zip version
  (**Settings → 7-Zip**);
- what you did, what you expected and what happened;
- the job’s **View log** output, if a job failed.

Security problems: please follow [Security](/docs/reference/security/#reporting-a-vulnerability)
instead of opening a public issue.

## Contributing code

1. Fork the repository and create a branch from `dev`.
2. Make your change, with tests where it makes sense. Keep 7-Zip logic in `crates/core`, update
   logic in `crates/updater`, and UI in `crates/app`.
3. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings` and
   `cargo test --workspace`.
4. Add a line under **Unreleased** in `CHANGELOG.md` for anything users will notice.
5. Open a pull request against `dev`. CI builds and tests it on Linux, macOS and Windows.

See [Building from source](/docs/project/building/) to get set up.

## Documentation

The docs are Markdown files in `website/src/content/docs/`. Every page has an **Edit page** link at
the bottom. Preview locally with `npm run dev` in `website/`.

## Ground rules

- Be kind; follow the project’s code of conduct.
- 7-Zip is never bundled, downloaded or linked by the app — that is a hard rule.
- No telemetry, no network requests beyond the update check.
