# abc-scaffold

**A project starter with a built-in AI development team.**

`abc-init` scaffolds a new project and drops in a `CLAUDE.md` that teaches Claude Code three roles — Amy (product manager), Bob (engineer), and Con (reviewer). Every task goes through the full clarify → implement → review cycle automatically, in one conversation.

---

## Install

```bash
cargo install --git https://github.com/Laotree/abc-scaffold
```

> **No Rust?** Use the shell script instead — see [Option C](#option-c--shell-script) below.

---

## Create a project

### Option A — Interactive TUI *(recommended)*

```bash
abc-init
```

A full-screen wizard walks you through two questions:

```
┌──────────────────────────────────────────────────────┐
│ 🚀 abc-init                                          │
│   Create a new abc-scaffold project                  │
├──────────────────────────────────────────────────────┤
│ Project name:                                        │
│ ╭────────────────────────────╮                       │
│ │ my-app_                    │                       │
│ ╰────────────────────────────╯                       │
│              Enter — continue  •  Esc — quit         │
└──────────────────────────────────────────────────────┘
```

```
┌──────────────────────────────────────────────────────┐
│ 🚀 abc-init                                          │
│   Create a new abc-scaffold project                  │
├──────────────────────────────────────────────────────┤
│ Language / stack  for my-app                         │
│ ╭──────────────────────────────────────────────────╮ │
│ │▶ 🦀 Rust           Systems programming           │ │
│ │  🐹 Go             Cloud-native services         │ │
│ │  🐍 Python         Scripting & ML                │ │
│ │  📘 TypeScript     Web & tooling                 │ │
│ ╰──────────────────────────────────────────────────╯ │
│   ↑/↓ navigate  •  Enter select  •  Esc back         │
└──────────────────────────────────────────────────────┘
```

A confirmation screen shows the summary before anything is written to disk.

### Option B — Partial or fully non-interactive

Skip prompts by passing arguments directly:

| Command | What it does |
|---------|-------------|
| `abc-init my-app` | Skips the name prompt; opens TUI at language select |
| `abc-init my-app --lang go` | Skips name + language; opens TUI at confirmation |
| `abc-init my-app --lang rust --yes` | Fully scripted — no prompts at all |

Available flags:

```
abc-init [NAME] [OPTIONS]

Arguments:
  [NAME]   Project name (skips name prompt)

Options:
  -l, --lang <LANG>  Language: rust | go | python | typescript
  -y, --yes          Skip confirmation prompt
  -h, --help         Print help
  -V, --version      Print version
```

### Option C — Shell script

No Rust compiler needed:

```bash
curl -sL https://raw.githubusercontent.com/Laotree/abc-scaffold/main/start.sh | bash -s my-app
```

---

## What gets scaffolded

Every project gets the same agent workflow files plus language-specific sources.

<details open>
<summary><b>🦀 Rust project</b></summary>

```
my-app/
├── CLAUDE.md          ← Amy/Bob/Con workflow + cargo Make commands
├── README.md
├── Makefile           ← build / release / test / lint / fmt / clean / hooks
├── .gitignore
├── hooks/
│   └── pre-push       ← blocks direct pushes to main
├── Cargo.toml         ← [package] name = "my-app", edition = "2021"
└── src/
    └── main.rs        ← fn main() starter
```

</details>

<details>
<summary><b>🐹 Go project</b></summary>

```
my-app/
├── CLAUDE.md          ← Amy/Bob/Con workflow + go Make commands
├── README.md
├── Makefile           ← build / test / lint / fmt / clean / hooks
├── .gitignore
├── hooks/
│   └── pre-push
├── go.mod             ← module my-app, go 1.22
└── main.go            ← package main starter
```

</details>

<details>
<summary><b>🐍 Python project</b></summary>

```
my-app/
├── CLAUDE.md
├── README.md
├── Makefile           ← install / run / test / lint / fmt / hooks
├── .gitignore
├── hooks/
│   └── pre-push
├── pyproject.toml     ← PEP 517, requires-python = ">=3.11"
└── main.py            ← def main() starter
```

</details>

<details>
<summary><b>📘 TypeScript project</b></summary>

```
my-app/
├── CLAUDE.md
├── README.md
├── Makefile           ← install / build / dev / test / lint / fmt / hooks
├── .gitignore
├── hooks/
│   └── pre-push
├── package.json       ← scripts: build, dev, test
├── tsconfig.json      ← strict, ES2022, outDir: dist
└── src/
    └── index.ts       ← function main() starter
```

</details>

A git repository is initialised automatically with a single `init from abc-scaffold` commit.

---

## The agent team

Once your project is created, open it in Claude Code. The `CLAUDE.md` teaches Claude three distinct roles that activate when you prefix your message with `@Amy`, `@Bob`, or `@Con` — or just describe a task and Amy will pick it up.

### Amy — Project Manager

Amy's job is to **prevent wasted effort**. She never lets Bob write a line of code based on a misunderstanding.

When you bring Amy a task, she asks clarifying questions until she fully understands: scope, edge cases, acceptance criteria. Once she's satisfied, she writes a clear task brief and immediately hands off to Bob — without waiting for you.

### Bob — Engineer

Bob picks up Amy's task brief and **implements it**. He works on a feature branch, writes or updates tests alongside the code, and opens a PR when done. He immediately hands off to Con — without waiting for you.

Bob never pushes directly to `main`. Every change goes through a PR.

### Con — Reviewer

Con is the **gatekeeper**. He reviews Bob's PR for correctness, style, and security, verifies tests pass, and either approves or requests changes. Con is the only one who may merge to `main`. After merging, he deletes the feature branch.

### The workflow

```
You describe a task
  └─▶ Amy clarifies until scope is locked
        └─▶ Bob implements on a feature branch + opens PR
              └─▶ Con reviews, approves, merges, cleans up
```

This all happens **in one conversation** — Amy hands to Bob hands to Con automatically. You describe, they execute.

### A typical session

```
@Amy I want to add a CLI flag --verbose that prints request details
```

Amy asks: *What counts as "request details"? Should it include headers? Response body?*

You answer, Amy confirms scope, Bob implements, Con reviews and merges — done.

---

## Branch protection

`hooks/pre-push` blocks direct pushes to `main` and `master`. Run `make hooks` after cloning to activate it. Con enforces the same rule in code review.

---

## License

MIT
