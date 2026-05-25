# abc-scaffold

A project starter built around the **Amy / Bob / Con** agent team. Use `abc-init` to create a new project interactively — pick your language, name your project, and the team handles the rest.

| Agent | Role |
|-------|------|
| **Amy** | Project Manager — clarifies requirements before any code is written |
| **Bob** | Engineer — implements on a feature branch, opens a PR |
| **Con** | Reviewer — reviews, approves, merges, cleans up |

## Quick Start

### Option A — Interactive TUI (recommended)

```bash
cargo install --git https://github.com/Laotree/abc-scaffold
abc-init
```

Arrow keys to pick your language, Enter to confirm — that's it.

### Option B — Non-interactive

```bash
abc-init my-app --lang go --yes      # fully scripted
abc-init my-app                      # TUI starting at language step
```

### Option C — Shell script (no Rust required)

```bash
curl -sL https://raw.githubusercontent.com/Laotree/abc-scaffold/main/start.sh | bash -s my-new-project
```

## What gets scaffolded

For every language you get the same ABC workflow files plus language-specific sources:

```
my-app/
├── CLAUDE.md          # Amy/Bob/Con agent instructions (language-aware Make commands)
├── README.md          # Project readme
├── Makefile           # build / test / lint / fmt / hooks targets
├── hooks/pre-push     # Blocks direct push to main
├── .gitignore
└── <language files>   # Cargo.toml + src/main.rs · go.mod + main.go
                       # pyproject.toml + main.py · package.json + src/index.ts
```

A fresh git repo with an `init from abc-scaffold` commit is created automatically.

## Supported stacks

| Flag | Files created |
|------|--------------|
| `--lang rust` *(default)* | `Cargo.toml`, `src/main.rs` |
| `--lang go` | `go.mod`, `main.go` |
| `--lang python` | `pyproject.toml`, `main.py` |
| `--lang typescript` | `package.json`, `tsconfig.json`, `src/index.ts` |

## After scaffolding

```bash
cd my-app
# open Claude Code and say:
@Amy I want to build ...
```

Amy clarifies your requirements, hands off to Bob for implementation, Bob hands off to Con for review. The full cycle runs in one conversation.

## Make Targets

| Target | Description |
|--------|-------------|
| `make build` | Build the project |
| `make test` | Run tests |
| `make fmt` | Format source |
| `make lint` | Lint |
| `make hooks` | Install git pre-push hook |

## Branch Protection

`hooks/pre-push` blocks direct pushes to `main` and `master`. Run `make hooks` to activate it.

## License

MIT
