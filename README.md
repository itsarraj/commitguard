# commitguard

A native Conventional Commits linter + changelog generator.
`commitlint`/`semantic-release` (Node) are the standard for this, and
their Node cold-start cost is a small, constant tax paid on literally
every single commit. This is a static binary — same job, no `npm install`.

## Usage

```bash
commitguard install-hook              # wires a commit-msg hook into .git/hooks
commitguard lint path/to/msgfile      # what the hook actually calls
commitguard changelog                 # full history, grouped by type
commitguard changelog --range v1.0.0..HEAD --version v1.1.0
```

## Rules enforced

Subject line must match `type(scope)!: description` (Conventional
Commits' own grammar) with `type` one of `feat fix docs style refactor
perf test build ci chore revert`; description starts lowercase and
doesn't end with a period; subject line ≤100 characters. A `BREAKING
CHANGE:` footer (or `!` after the type/scope) marks a commit as breaking
either way — checked independently, not just presence of the `!`.

## Changelog grouping

`feat` → Features, `fix` → Bug Fixes, `perf` → Performance, `revert` →
Reverts, everything else → Other — in that reader-priority order, not
alphabetical or type-declaration order. Breaking commits get a
`BREAKING CHANGES` section at the top *in addition to* their normal
section, so both "what changed" and "here's everything that shipped in
this release" stay answerable from the same document.

## Status: built and verified with a real commit genuinely blocked, then a real changelog from real history

- **21 unit tests** (`cargo test --lib`): the Conventional Commits
  parser (scope, the `!` breaking marker, a `BREAKING CHANGE:` footer
  correctly marking breaking *even without* the `!`, multi-paragraph
  bodies, and a non-conventional subject producing a descriptive error
  that names the actual bad text rather than just failing silently); every
  lint rule individually and a message failing three rules at once, to
  confirm they don't short-circuit each other; changelog grouping and
  section ordering (a stable, reader-priority order, not insertion or
  alphabetical); and — using a **real git repository**, not fixtures —
  `commit_messages` retrieving history in chronological order, a
  **multi-paragraph body with blank lines correctly staying one commit
  message** (the actual reason this uses `\x1e` as a separator instead of
  a blank line, verified not just asserted), and a revision range
  (`v1.0.0..HEAD` against a real tag) correctly limiting what comes back.
- **Full CLI run through a real commit sequence**: installed the real
  `commit-msg` hook in a throwaway repo, attempted `git commit -m "fixed
  the login bug"` — **genuinely rejected**, the hook's own rule output
  printed as the reason, exit code 1; committed `fix: correct login
  redirect after session expiry` — **went through**; added a `feat(api):`
  commit and a `feat(api)!:` commit with a `BREAKING CHANGE:` footer; ran
  `commitguard changelog` for real against that actual history and got
  back a correctly-grouped, correctly-sectioned Markdown changelog with
  the breaking change called out separately, generated from real
  `git log` output, not a hand-fed fixture.

**Not done / deliberately deferred**: a config file for customizing the
allowed-types list or disabling individual rules (`commitlint`'s
`.commitlintrc` equivalent — this v1's rule set is fixed, which is fine
for a single-repo tool but a real gap for anyone wanting a different
convention); scope validation against a fixed allowlist (any string in
parens is accepted as a scope today); and `--amend`-aware changelog
regeneration (rewriting `CHANGELOG.md` in place vs. just printing to
stdout for the caller to redirect).
