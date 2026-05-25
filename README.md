# abc-scaffold

A project starter built around the **Amy / Bob / Con** agent team. Clone it, start a new project, and let the team handle the development workflow.

| Agent | Role |
|-------|------|
| **Amy** | Project Manager — clarifies requirements before any code is written |
| **Bob** | Engineer — implements on a feature branch, opens a PR |
| **Con** | Reviewer — reviews, approves, merges, cleans up |

## Quick Start

```bash
./start.sh my-new-project
cd my-new-project
```

Or from anywhere via curl:

```bash
curl -sL https://raw.githubusercontent.com/Laotree/abc-scaffold/main/start.sh | bash -s my-new-project
cd my-new-project
```

That's it — clean git repo, project renamed, ready to go. Open Claude Code and say:

```
@Amy I want to build ...
```

Amy clarifies your requirements, hands off to Bob for implementation, Bob hands off to Con for review. The full cycle runs in one conversation.

## What's Included

```
abc-scaffold/
├── start.sh                         # One command to create a new project
├── CLAUDE.md                        # Agent workflow — Amy/Bob/Con instructions
├── Makefile                         # build / test / lint / fmt / hooks
├── hooks/pre-push                   # Blocks direct push to main (PRs only)
├── .github/workflows/release.yml    # CI template (commented out, adapt to your stack)
├── Cargo.toml                       # Rust default (replace for other languages)
├── src/main.rs                      # Placeholder entry point
├── LICENSE                          # MIT
└── .gitignore
```

## Adapting to Your Stack

The scaffold defaults to Rust. To use a different language:

1. Replace `Cargo.toml` and `src/` with your language's project config and source
2. Update the `Makefile` targets (`build`, `test`, `lint`, `fmt`) to match your toolchain
3. Uncomment and adapt `.github/workflows/release.yml` for your CI/CD
4. Update the **Commands** section in `CLAUDE.md`

The agent workflow in `CLAUDE.md` is language-agnostic — it works regardless of your stack.

## Make Targets

| Target | Description |
|--------|-------------|
| `make build` | Debug build |
| `make release` | Release build |
| `make test` | Run tests |
| `make fmt` | Format source |
| `make lint` | Lint (clippy) |
| `make clean` | Remove build artifacts |
| `make hooks` | Install git pre-push hook |

## Branch Protection

The `hooks/pre-push` hook blocks direct pushes to `main` and `master`. All changes go through feature branches and PRs — enforced by the hook and by Con's review rules.

Run `make hooks` after cloning to activate it.

## License

MIT
