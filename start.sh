#!/usr/bin/env bash
set -euo pipefail

NAME="${1:?Usage: ./start.sh <project-name>}"
REPO="https://github.com/Laotree/abc-scaffold.git"

if [[ -d "$NAME" ]]; then
    echo "Directory '$NAME' already exists."
    exit 1
fi

echo "-> Creating $NAME from abc-scaffold..."

git clone --depth 1 "$REPO" "$NAME" 2>/dev/null
rm -rf "$NAME/.git" "$NAME/start.sh"

cd "$NAME"

if [[ "$NAME" != "my-project" ]]; then
    sed -i '' "s/my-project/$NAME/g" Cargo.toml 2>/dev/null || sed -i "s/my-project/$NAME/g" Cargo.toml
    sed -i '' "s/my-project/$NAME/g" src/main.rs 2>/dev/null || sed -i "s/my-project/$NAME/g" src/main.rs
fi

git init -b main
git add -A
git commit -m "init from abc-scaffold"

echo ""
echo "Done. To start:"
echo "  cd $NAME"
echo "  # call @Amy with your first task"
