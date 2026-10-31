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

## Conventions

- `cargo clippy --all-targets` must be clean (`all` + `pedantic` are enabled in `Cargo.toml`).
