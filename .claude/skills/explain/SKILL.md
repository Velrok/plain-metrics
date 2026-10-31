---
name: explain
description: |
  Create, update and list the maintainer's Rust lessons in `lessons/` (personal learning notes, not user docs). Use when: the user asks to "explain X" for a lesson, "add/update a lesson", "write a tutorial on <Rust topic>", or /explain is invoked.
user-invocable: true
argument-hint: "[topic, or 'list']"
---

# Explain

Lessons in `lessons/` teach the maintainer Rust. The rules (audience, std-first, crate comparison, heavy what/why comments, no CLI docs) live in the **Lessons** section of `CLAUDE.md`. Read it first and follow it.

## Workflow

1. `ls lessons/` and `rg -il <keyword> lessons/`. Update the existing lesson when the topic is already covered; create a new file only for a new topic.
2. Name the file `lessons/<topic-in-kebab-case>.md`.
3. Draft with the layout below.
4. Run `scripts/check.sh lessons/<file>.md`: it compiles and clippy-checks every `rust` block.
5. Run the code for real (e.g. send a message with `nc -U`) and say in the lesson that it was run.
6. Report the path. Commit only when asked (`git-commit-expert`).

`/explain list` prints the files in `lessons/` with their first heading.

## Layout

```markdown
# <Title> (<option A> vs <option B> when a crate competes)

> 📘 Personal lesson, not project docs. <what was compiled/run>

## 🧠 Mental model        bullets + a type/role table
## 💻 Option A: std only  heavily commented code
## 🔍 Walkthrough         the non-obvious lines
## ▶️ Try it              commands to run it
## 💻 Option B: <crate>   only when a crate is more idiomatic; its `Cargo.toml` deps in a ```toml block
## ⚖️ A vs B              table (deps, scaling, shutdown, learning curve)
## 🧭 Which to pick       recommendation for plain-metrics
## ⚠️ Gotchas
## ➡️ Next steps          what to build next in this repo
```

Drop sections that don't apply. Keep tables under 100 characters wide.

## Comment style in lesson code

Every comment gives the **what first, then the why**:

```rust
// What: delete the socket file if it already exists.
// Why: `bind` fails with "address in use" if a crashed run left it behind.
```

## Checker

`scripts/check.sh <lesson.md>` builds each `rust` block as its own cargo project, adding any dependency lines from the lesson's ```toml block. It runs `cargo clippy` with pedantic lints and prints one line per block (`ok` or the compiler errors). It does not run the code.
