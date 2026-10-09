# Contributing to Mosaic

<sub>[Español](CONTRIBUTING.md)</sub>

Thanks for wanting to help. Mosaic is small and wants to stay that way, so this
guide is short.

## Before you start

Mosaic has a few principles that aren't up for debate. A proposal that breaks
one won't be merged, however good it is:

- **Local.** No network requests at all. No telemetry, no analytics, no update
  checks, no linked fonts.
- **No AI.** No model APIs.
- **Read-only on your repositories.** Mosaic reads metadata and never writes to
  a project folder or runs Git commands with side effects.
- **Nothing gets deleted.** A project that disappears is marked as missing, not
  removed, so its tags and notes aren't lost.
- **Fast with hundreds of projects.**

For anything bigger than a fix, open an issue first so we can talk it through.
It saves both of us work.

## Setting up

You need stable Rust, Node 20 or later, [pnpm](https://pnpm.io) and [the Tauri 2
dependencies](https://tauri.app/start/prerequisites/) for your system. The
README lists them for Fedora.

```bash
pnpm install
MOSAIC_LOG=debug pnpm tauri dev
```

The development database is the same one the installed app uses. To test
without touching yours, on Linux you can point it at another folder:

```bash
XDG_DATA_HOME=/tmp/mosaic-test pnpm tauri dev
```

## Where things go

```
views/ → stores/ → api/ → invoke()  ⇄  commands/ → core/ → db/
```

- `src-tauri/src/core/`: the logic, with no Tauri in it. Unit tests go here.
- `src-tauri/src/db/repositories/`: all the SQL, and only here.
- `src-tauri/src/commands/`: validates input and delegates. No logic.
- `src/lib/api/`: the only place in the frontend that calls `invoke()`.
- `src/lib/types/index.ts`: a literal copy of the Rust structs. Change one,
  change the other.
- `src/lib/styles/tokens.css`: every color, size and spacing. Components don't
  hard-code values.

More detail in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), and the reasoning
behind the design in [`docs/DESIGN.md`](docs/DESIGN.md) (both in Spanish).

## Before opening a pull request

The same checks CI runs on Linux, macOS and Windows:

```bash
pnpm check            # types and templates, fails on warnings too
pnpm test             # frontend tests
pnpm build

cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Also:

- **A change in behavior comes with a test.** The non-obvious rules, like a
  project's timestamps or what counts as a project, are the most tested.
- **No new dependencies without discussing them first.** Each one is code we
  don't control.
- **Never `unwrap()` outside tests**, and errors go through `AppError`.
- **Comments only for the why.** One line of docs on public items and, inside
  the code, only what explains a decision you can't see by reading it.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/), small, one topic
per commit. The history is in Spanish, but English is welcome.

```
feat(ui): tooltip on the card name
fix(core): don't mark projects under a disabled folder as missing
```

## Reporting a bug

Open an issue with your operating system, Mosaic version, what you did, what
you expected and what happened. If you can, add the output of running it with
`MOSAIC_LOG=debug`.

## License

By contributing you agree that your contribution is published under the
project's [Apache 2.0 license](LICENSE).
