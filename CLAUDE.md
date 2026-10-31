# plain-metrics

Float-only metrics server plus `metrics` CLI. See `README.md` for the interface.

## ⚠️ Decision log: `docs/decisions.md`

**Keep it up to date.** It is the source of truth for why the project looks the way it does.

- If the `keep-decision-log` skill is available, load it. It is user-invoked (`/keep-decision-log`), so ask the user to run it if you can't. This repo's log is `docs/decisions.md` (in the repo, not the notes vault), so the skill's vault steps don't apply.
- Add an entry for every **decision**, finding, or user correction **as it happens**, not at the end.
- A decision is a choice between real alternatives that changes behaviour, shape or promises, with a reason. **Admin tasks are not decisions**: renames, regenerating docs, README tweaks that follow a logged decision, commits. Don't log them.
- Number entries `D1`, `D2`, … in order. Next number = last + 1. Numbers are never reused; gaps are expected.
- **Never rewrite an old entry.** Add a new one that says which earlier entry it supersedes.
- Keep each entry to 1–3 lines: what, why, evidence.
- Move answered questions out of "Open questions"; add new ones as they come up.
- Read the log before changing behaviour, so you don't re-litigate settled decisions.

## Keep docs in sync

- `docs/cli-help.md` holds `--help` for `metrics`, `server` and `record`. Regenerate it whenever CLI args change.
- Update `README.md` when the user-facing interface changes.

## Lessons

- `lessons/` is for the **maintainer's own learning**, not end-user docs. One markdown file per topic, explaining a Rust concept or technique.
- Prefer std-only, runnable examples. Where an established crate is the more idiomatic choice, show it too and add a std-vs-crate comparison table (deps, scaling, trade-offs) plus a recommendation for this project.
- Compile and run all code before including it.
- Annotate lesson code **heavily** with comments, unlike production code: give each comment the **what first, then the why** (a "What:" sentence followed by a "Why:" sentence). Cover ownership, `?`, `move`, `.await` and why each call exists. The reader is learning Rust.
- Don't document the project's CLI there; that belongs in `README.md` and `docs/cli-help.md`.

## Conventions

- `cargo clippy --all-targets` must be clean (`all` + `pedantic` are enabled in `Cargo.toml`).
