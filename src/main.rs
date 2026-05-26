//! abc-init — Interactive TUI wizard to scaffold abc-scaffold projects.
//!
//! Usage:
//!   abc-init                             # full TUI wizard
//!   abc-init my-app                      # skip name prompt
//!   abc-init my-app --lang go            # skip name + language prompts
//!   abc-init my-app --lang rust --yes    # fully non-interactive

use std::{
    fs,
    io::{self, Stdout},
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "abc-init",
    version,
    about = "Interactive TUI wizard — scaffold a new abc-scaffold project"
)]
struct Cli {
    /// Project name (skips the name prompt)
    name: Option<String>,

    /// Language / stack: rust | go | python | typescript
    #[arg(short, long, value_enum)]
    lang: Option<LangChoice>,

    /// Skip the confirmation prompt (auto-yes)
    #[arg(short, long)]
    yes: bool,
}

#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
enum LangChoice {
    Rust,
    Go,
    Python,
    Typescript,
}

// ─── Language enum ────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum Lang {
    Rust,
    Go,
    Python,
    TypeScript,
}

impl Lang {
    fn label(&self) -> &'static str {
        match self {
            Lang::Rust => "Rust",
            Lang::Go => "Go",
            Lang::Python => "Python",
            Lang::TypeScript => "TypeScript",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Lang::Rust => "Systems programming  —  cargo, Cargo.toml, src/main.rs",
            Lang::Go => "Cloud-native services  —  go mod, go.mod, main.go",
            Lang::Python => "Scripting & ML  —  uv/pip, pyproject.toml, main.py",
            Lang::TypeScript => "Web & tooling  —  npm, package.json, src/index.ts",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            Lang::Rust => "🦀",
            Lang::Go => "🐹",
            Lang::Python => "🐍",
            Lang::TypeScript => "📘",
        }
    }
}

const ALL_LANGS: &[Lang] = &[Lang::Rust, Lang::Go, Lang::Python, Lang::TypeScript];

fn lang_index(lc: &LangChoice) -> usize {
    match lc {
        LangChoice::Rust => 0,
        LangChoice::Go => 1,
        LangChoice::Python => 2,
        LangChoice::Typescript => 3,
    }
}

// ─── App state ────────────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum Screen {
    NameInput,
    LangSelect,
    Confirm,
    Done,
}

struct App {
    screen: Screen,
    name: String,
    lang_state: ListState,
    error: Option<String>,
    /// Message shown on the Done screen (and briefly during scaffolding).
    done_msg: String,
    quit: bool,
}

impl App {
    fn new(initial_name: Option<String>, initial_lang: Option<usize>) -> Self {
        let screen = match (&initial_name, initial_lang) {
            (Some(_), Some(_)) => Screen::Confirm,
            (Some(_), None) => Screen::LangSelect,
            _ => Screen::NameInput,
        };
        let mut lang_state = ListState::default();
        lang_state.select(Some(initial_lang.unwrap_or(0)));
        Self {
            screen,
            name: initial_name.unwrap_or_default(),
            lang_state,
            error: None,
            done_msg: String::new(),
            quit: false,
        }
    }

    fn selected_lang(&self) -> &Lang {
        &ALL_LANGS[self.lang_state.selected().unwrap_or(0)]
    }

    fn lang_idx(&self) -> usize {
        self.lang_state.selected().unwrap_or(0)
    }

    fn move_up(&mut self) {
        let i = self.lang_idx();
        self.lang_state
            .select(Some(if i == 0 { ALL_LANGS.len() - 1 } else { i - 1 }));
    }

    fn move_down(&mut self) {
        let i = self.lang_idx();
        self.lang_state.select(Some((i + 1) % ALL_LANGS.len()));
    }
}

// ─── Terminal helpers ─────────────────────────────────────────────────────────

type Term = Terminal<CrosstermBackend<Stdout>>;

fn setup() -> io::Result<Term> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(io::stdout()))
}

fn teardown(term: &mut Term) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()
}

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let initial_lang = cli.lang.as_ref().map(lang_index);

    // Fully non-interactive path
    if let (Some(name), Some(lang_idx), true) = (&cli.name, initial_lang, cli.yes) {
        let lang = &ALL_LANGS[lang_idx];
        println!("Creating {} ({})…", name, lang.label());
        scaffold(name, lang)?;
        println!(
            "✅  Done!  cd {}  and call @Amy with your first task.",
            name
        );
        return Ok(());
    }

    // TUI path
    let mut app = App::new(cli.name.clone(), initial_lang);
    let mut term = setup()?;
    let result = run(&mut term, &mut app);
    teardown(&mut term)?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    // Print next-steps after TUI closes so the message survives in the scrollback
    if app.screen == Screen::Done && !app.done_msg.is_empty() {
        println!("\n{}", app.done_msg);
    }

    Ok(())
}

// ─── Event loop ───────────────────────────────────────────────────────────────

fn run(term: &mut Term, app: &mut App) -> io::Result<()> {
    loop {
        term.draw(|f| ui(f, app))?;

        if app.quit {
            break;
        }

        if let Event::Key(key) = event::read()? {
            // Global emergency exit
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.quit = true;
                break;
            }

            match app.screen {
                Screen::NameInput => handle_name_input(app, key.code),
                Screen::LangSelect => handle_lang_select(app, key.code),
                Screen::Confirm => handle_confirm(app, term, key.code)?,
                Screen::Done => {
                    app.quit = true; // any key exits
                }
            }
        }
    }
    Ok(())
}

// ─── Input handlers ───────────────────────────────────────────────────────────

fn handle_name_input(app: &mut App, key: KeyCode) {
    app.error = None;
    match key {
        KeyCode::Char(c) => app.name.push(c),
        KeyCode::Backspace => {
            app.name.pop();
        }
        KeyCode::Enter => {
            let name = app.name.trim().to_string();
            if name.is_empty() {
                app.error = Some("Project name cannot be empty.".into());
            } else if name.contains('/') || name.contains('\\') {
                app.error = Some("Project name cannot contain slashes.".into());
            } else {
                app.name = name;
                app.screen = Screen::LangSelect;
            }
        }
        KeyCode::Esc => app.quit = true,
        _ => {}
    }
}

fn handle_lang_select(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Enter => app.screen = Screen::Confirm,
        KeyCode::Esc => app.screen = Screen::NameInput,
        _ => {}
    }
}

fn handle_confirm(app: &mut App, term: &mut Term, key: KeyCode) -> io::Result<()> {
    match key {
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Show a "working…" frame before blocking on disk I/O
            app.done_msg = format!("  Creating {}…", app.name);
            term.draw(|f| ui(f, app))?;
            app.done_msg.clear();

            let name = app.name.clone();
            let lang = app.selected_lang().clone();

            match scaffold(&name, &lang) {
                Ok(()) => {
                    app.done_msg = format!(
                        "✅  Project '{}' created!\n\n  cd {}\n  # call @Amy with your first task",
                        name, name
                    );
                    app.screen = Screen::Done;
                }
                Err(e) => {
                    app.error = Some(e.to_string());
                    // Stay on Confirm so the user can try again or go back
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.error = None;
            app.screen = Screen::LangSelect;
        }
        _ => {}
    }
    Ok(())
}

// ─── UI renderer ─────────────────────────────────────────────────────────────

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    // Outer chrome
    let outer = Block::default()
        .title(Span::styled(
            " 🚀 abc-init ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(outer, area);

    // Three vertical zones inside the chrome
    let zones = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1), // subtitle
            Constraint::Min(0),    // content
            Constraint::Length(1), // help / error bar
        ])
        .split(area);

    // Subtitle
    f.render_widget(
        Paragraph::new(Span::styled(
            "Create a new abc-scaffold project",
            Style::default().fg(Color::Gray),
        ))
        .alignment(Alignment::Center),
        zones[0],
    );

    // Content
    match &app.screen {
        Screen::NameInput => render_name_input(f, zones[1], app),
        Screen::LangSelect => render_lang_select(f, zones[1], app),
        Screen::Confirm => render_confirm(f, zones[1], app),
        Screen::Done => render_done(f, zones[1], app),
    }

    // Bottom bar: error takes priority over hints
    if let Some(err) = &app.error {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("⚠  ", Style::default().fg(Color::Red)),
                Span::styled(err.as_str(), Style::default().fg(Color::Red)),
            ])),
            zones[2],
        );
    } else {
        let hints = match app.screen {
            Screen::NameInput => "Enter — continue  •  Esc — quit",
            Screen::LangSelect => "↑/↓ or j/k — navigate  •  Enter — select  •  Esc — back",
            Screen::Confirm => "Enter / Y — create  •  N / Esc — back",
            Screen::Done => "Any key — exit",
        };
        f.render_widget(
            Paragraph::new(Span::styled(hints, Style::default().fg(Color::DarkGray)))
                .alignment(Alignment::Center),
            zones[2],
        );
    }
}

fn render_name_input(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    f.render_widget(
        Paragraph::new(Span::styled(
            "Project name:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        chunks[0],
    );

    // Position the real terminal cursor at the end of the input
    f.set_cursor_position((chunks[1].x + 1 + app.name.len() as u16, chunks[1].y + 1));

    f.render_widget(
        Paragraph::new(app.name.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White)),
        chunks[1],
    );
}

fn render_lang_select(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "Language / stack  ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("for ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.name.as_str(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        chunks[0],
    );

    let items: Vec<ListItem> = ALL_LANGS
        .iter()
        .map(|l| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("  {} {:<14}", l.emoji(), l.label())),
                Span::styled(l.description(), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = app.lang_state.clone();
    f.render_stateful_widget(list, chunks[1], &mut state);
}

fn render_confirm(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    // Show the transient "working…" message if set
    if !app.done_msg.is_empty() {
        f.render_widget(
            Paragraph::new(app.done_msg.as_str())
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let lang = app.selected_lang();
    let summary = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Name:      ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.name.as_str(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Language:  ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{} {}", lang.emoji(), lang.label())),
        ]),
        Line::from(vec![
            Span::styled("  Directory: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("./{}", app.name), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Ready to scaffold. Press Enter (or Y) to create.",
            Style::default().fg(Color::White),
        )]),
    ];

    f.render_widget(
        Paragraph::new(summary)
            .block(
                Block::default()
                    .title(" Summary ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_done(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    f.render_widget(
        Paragraph::new(app.done_msg.as_str())
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        area,
    );
}

// ─── Scaffolding ──────────────────────────────────────────────────────────────

fn scaffold(name: &str, lang: &Lang) -> io::Result<()> {
    let dir = PathBuf::from(name);

    if dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "Directory '{}' already exists — choose a different name or remove it first.",
                name
            ),
        ));
    }

    fs::create_dir_all(&dir)?;
    fs::create_dir_all(dir.join("hooks"))?;

    // Common files
    write_file(&dir, "CLAUDE.md", &tmpl_claude_md(lang))?;
    write_file(&dir, "README.md", &tmpl_readme(name, lang))?;
    write_file(&dir, ".gitignore", tmpl_gitignore(lang))?;
    write_file(&dir, "Makefile", &tmpl_makefile(lang, name))?;
    write_file(&dir, "hooks/pre-push", TMPL_PRE_PUSH)?;
    make_executable(&dir.join("hooks/pre-push"))?;

    // Language-specific files
    match lang {
        Lang::Rust => scaffold_rust(&dir, name)?,
        Lang::Go => scaffold_go(&dir, name)?,
        Lang::Python => scaffold_python(&dir, name)?,
        Lang::TypeScript => scaffold_typescript(&dir, name)?,
    }

    // Initial git commit
    run_git(&dir, &["init", "-b", "main"])?;
    run_git(&dir, &["add", "-A"])?;
    run_git(&dir, &["commit", "-m", "init from abc-scaffold"])?;

    Ok(())
}

fn write_file(base: &Path, rel: &str, content: &str) -> io::Result<()> {
    let path = base.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn make_executable(_: &Path) -> io::Result<()> {
    Ok(())
}

fn run_git(dir: &Path, args: &[&str]) -> io::Result<()> {
    let status = Command::new("git").current_dir(dir).args(args).status()?;
    if !status.success() {
        return Err(io::Error::other(format!("git {} failed", args.join(" "))));
    }
    Ok(())
}

// ─── Language scaffolders ─────────────────────────────────────────────────────

fn scaffold_rust(dir: &Path, name: &str) -> io::Result<()> {
    write_file(dir, "Cargo.toml", &tmpl_rust_cargo(name))?;
    write_file(dir, "src/main.rs", TMPL_RUST_MAIN)
}

fn scaffold_go(dir: &Path, name: &str) -> io::Result<()> {
    write_file(dir, "go.mod", &tmpl_go_mod(name))?;
    write_file(dir, "main.go", &tmpl_go_main(name))
}

fn scaffold_python(dir: &Path, name: &str) -> io::Result<()> {
    write_file(dir, "pyproject.toml", &tmpl_python_pyproject(name))?;
    write_file(dir, "main.py", TMPL_PYTHON_MAIN)
}

fn scaffold_typescript(dir: &Path, name: &str) -> io::Result<()> {
    write_file(dir, "package.json", &tmpl_ts_package_json(name))?;
    write_file(dir, "tsconfig.json", TMPL_TS_CONFIG)?;
    write_file(dir, "src/index.ts", TMPL_TS_MAIN)
}

// ─── Templates ────────────────────────────────────────────────────────────────

fn tmpl_claude_md(lang: &Lang) -> String {
    let commands = match lang {
        Lang::Rust => {
            r#"```bash
make build       # debug build
make release     # release build
make test        # run tests
make lint        # clippy
make fmt         # format source
make clean       # remove build artifacts
make hooks       # install git pre-push hook
```"#
        }
        Lang::Go => {
            r#"```bash
make build       # go build
make test        # go test ./...
make lint        # golangci-lint run
make fmt         # gofmt -w .
make clean       # remove build artifacts
make hooks       # install git pre-push hook
```"#
        }
        Lang::Python => {
            r#"```bash
make install     # pip install -e ".[dev]"
make run         # python main.py
make test        # pytest
make lint        # ruff check .
make fmt         # ruff format .
make hooks       # install git pre-push hook
```"#
        }
        Lang::TypeScript => {
            r#"```bash
make install     # npm install
make build       # tsc
make dev         # ts-node src/index.ts
make test        # jest
make lint        # eslint src
make fmt         # prettier --write src
make hooks       # install git pre-push hook
```"#
        }
    };

    let arch = match lang {
        Lang::Rust => "Rust binary. Entry point `src/main.rs`. Replace with whatever your project needs.",
        Lang::Go => "Go binary. Entry point `main.go`. Replace with whatever your project needs.",
        Lang::Python => "Python application. Entry point `main.py`. Replace with whatever your project needs.",
        Lang::TypeScript => "TypeScript application. Entry point `src/index.ts`. Replace with whatever your project needs.",
    };

    format!(
        r#"# CLAUDE.md

abc-scaffold provides the Amy/Bob/Con agent team for any project. The workflow below is the core — the build tooling is just a default starting point.

## Commands

{commands}

## Architecture

{arch}

## Agents

### Amy — Project Manager

Amy ensures no code gets written based on a misunderstanding.

**Responsibilities:**
- Engage the user with clarifying questions until the request is fully understood
- Confirm scope, acceptance criteria, and edge cases before any code work begins
- Once understanding is confirmed, describe the task clearly

**When to invoke:** Any time a new feature request, bug report, or task arrives.

**Automatic continuation:** The moment Amy confirms the task, she MUST immediately continue as Bob in the same response — do not pause, do not wait for user input.

---

### Bob — Engineer

Bob implements what's been scoped.

**Responsibilities:**
- Pick up tasks scoped by Amy
- Implement following existing code conventions and architecture
- Write or update tests alongside the code
- Keep commits focused and message them clearly
- Always work on a feature branch and open a PR

**When to invoke:** After Amy has scoped a task.

**Automatic continuation:** The moment Bob finishes implementation, he MUST immediately continue as Con in the same response — do not pause, do not wait for user input.

**Hard rules:**
- NEVER push directly to main — all changes go through PRs
- Always work on a feature branch and open a PR
- PR must reference the issue/task it addresses

---

### Con — Reviewer

Con is the gatekeeper before anything merges.

**Responsibilities:**
- Review Bob's changes for correctness, style, and security
- Verify that all tests pass
- If criteria are met: approve; otherwise request changes
- Once approved and merged: clean up the feature branch

**Hard rules:**
- Con is the ONLY one who may merge to main
- Con must NEVER push directly to main
- Con must not merge until Amy (scope match) and Con (code quality) have approved

---

## Workflow

```
Amy clarifies -> Amy confirms -> Bob implements -> Con reviews -> Con merges + cleans up
```
"#
    )
}

fn tmpl_readme(name: &str, lang: &Lang) -> String {
    format!(
        r#"# {name}

> Scaffolded with [abc-scaffold](https://github.com/Laotree/abc-scaffold) · {emoji} {lang}

## Getting started

```bash
# call @Amy with your first task
```

## Team

| Agent | Role |
|-------|------|
| **Amy** | Project Manager — clarifies scope before any code is written |
| **Bob** | Engineer — implements what Amy scoped |
| **Con** | Reviewer — reviews, approves, and merges |
"#,
        name = name,
        emoji = lang.emoji(),
        lang = lang.label(),
    )
}

fn tmpl_gitignore(lang: &Lang) -> &'static str {
    match lang {
        Lang::Rust => "/target\n*.swp\n.DS_Store\n",
        Lang::Go => "# Binaries\n*.exe\n*.out\n/dist\n.DS_Store\n",
        Lang::Python => "__pycache__/\n*.py[cod]\n.venv/\ndist/\n.DS_Store\n",
        Lang::TypeScript => "node_modules/\ndist/\n*.js.map\n.DS_Store\n",
    }
}

fn tmpl_makefile(lang: &Lang, name: &str) -> String {
    match lang {
        Lang::Rust => r#".PHONY: build release test lint fmt clean hooks

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

clean:
	cargo clean

hooks:
	cp hooks/pre-push .git/hooks/pre-push
	chmod +x .git/hooks/pre-push
"#
        .into(),

        Lang::Go => format!(
            r#".PHONY: build test lint fmt clean hooks

build:
	go build -o {name} .

test:
	go test ./...

lint:
	golangci-lint run

fmt:
	gofmt -w .

clean:
	rm -f {name}

hooks:
	cp hooks/pre-push .git/hooks/pre-push
	chmod +x .git/hooks/pre-push
"#
        ),

        Lang::Python => r#".PHONY: install run test lint fmt hooks

install:
	pip install -e ".[dev]"

run:
	python main.py

test:
	pytest

lint:
	ruff check .

fmt:
	ruff format .

hooks:
	cp hooks/pre-push .git/hooks/pre-push
	chmod +x .git/hooks/pre-push
"#
        .into(),

        Lang::TypeScript => r#".PHONY: install build dev test lint fmt hooks

install:
	npm install

build:
	npx tsc

dev:
	npx ts-node src/index.ts

test:
	npx jest

lint:
	npx eslint src

fmt:
	npx prettier --write src

hooks:
	cp hooks/pre-push .git/hooks/pre-push
	chmod +x .git/hooks/pre-push
"#
        .into(),
    }
}

const TMPL_PRE_PUSH: &str = r#"#!/usr/bin/env bash
# Installed by: make hooks
set -euo pipefail
echo "pre-push: checks passed"
"#;

fn tmpl_rust_cargo(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
description = "TODO: describe your project"
license = "MIT"

[[bin]]
name = "{name}"
path = "src/main.rs"

[dependencies]
"#
    )
}

const TMPL_RUST_MAIN: &str = r#"fn main() {
    println!("Hello from your new project!");
}
"#;

fn tmpl_go_mod(name: &str) -> String {
    format!(
        r#"module {name}

go 1.22
"#
    )
}

fn tmpl_go_main(name: &str) -> String {
    format!(
        r#"package main

import "fmt"

func main() {{
    fmt.Println("Hello from {name}!")
}}
"#
    )
}

fn tmpl_python_pyproject(name: &str) -> String {
    format!(
        r#"[build-system]
requires = ["setuptools>=68"]
build-backend = "setuptools.backends.legacy:build"

[project]
name = "{name}"
version = "0.1.0"
requires-python = ">=3.11"
dependencies = []

[project.optional-dependencies]
dev = ["pytest", "ruff"]
"#
    )
}

const TMPL_PYTHON_MAIN: &str = r#"def main() -> None:
    print("Hello from your new project!")


if __name__ == "__main__":
    main()
"#;

fn tmpl_ts_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "build": "tsc",
    "dev": "ts-node src/index.ts",
    "test": "jest"
  }},
  "devDependencies": {{
    "typescript": "^5",
    "@types/node": "^20",
    "ts-node": "^10",
    "jest": "^29",
    "@types/jest": "^29"
  }}
}}
"#
    )
}

const TMPL_TS_CONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "commonjs",
    "lib": ["ES2022"],
    "outDir": "dist",
    "rootDir": "src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src"],
  "exclude": ["node_modules", "dist"]
}
"#;

const TMPL_TS_MAIN: &str = r#"function main(): void {
  console.log("Hello from your new project!");
}

main();
"#;
