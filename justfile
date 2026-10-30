# ClipKit Studio task runner. `just` lists recipes.
set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Format check, clippy on the whole workspace, fast tests, hook tests. Run after every step.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --message-format short --quiet -- -D warnings
    cargo nextest run --workspace --no-tests=pass --cargo-quiet
    tests/hooks/run.sh
    just studio-check

# Studio: prettier check, strict typecheck and unit tests (installs locked deps if missing).
studio-check:
    [ -d studio/node_modules ] || pnpm --dir studio install --frozen-lockfile --silent
    pnpm --dir studio --silent format:check
    pnpm --dir studio --silent typecheck
    pnpm --dir studio --silent test

# Build the studio into studio/dist (embedded by ck; debug builds read it from disk).
studio-build:
    [ -d studio/node_modules ] || pnpm --dir studio install --frozen-lockfile --silent
    pnpm --dir studio --silent build

# Format check plus clippy and unit tests of changed crates only (used by the Stop hook).
check-fast:
    scripts/check-fast.sh

# Full test suite (grows each phase: integration, contract, e2e).
test:
    cargo nextest run --workspace --no-tests=pass --cargo-quiet
    tests/hooks/run.sh

# Format all Rust code.
fmt:
    cargo fmt --all

# Regenerate the studio's TS bindings from the Rust types (`just check` fails on drift).
types:
    CK_WRITE_TYPES=1 cargo nextest run -p ck-server --cargo-quiet -E 'test(ts_types)'

# Studio dev server with hot reload, proxying /api and /ws to the `ck studio` running here.
# Open the printed Vite URL with `#token=<token from .clipkit/server.json>`.
studio-dev:
    CK_PORT="$(jq -r .port .clipkit/server.json)" pnpm --dir studio dev

# Playwright browser tests against a real `ck studio` per test (spec 10.1). Needs Chromium once:
# `pnpm --dir studio exec playwright install chromium`.
e2e: studio-build
    cargo build -p ck-cli --quiet
    pnpm --dir studio --silent e2e

# Generate synthetic fixture media and ground truth into fixtures/generated/.
fixtures out="fixtures/generated":
    fixtures/generate.sh {{out}}

# With/without-plugin benchmark (Phase 9; costs tokens, run on demand).
bench:
    @echo "PLACEHOLDER: bench harness (A0, A1, B, ablations) arrives in Phase 9 (spec 10.3)"; exit 1

# Re-split docs/MASTER_PROMPT.md into docs/spec/ (only when the master prompt changes).
split-spec:
    python3 -I scripts/split-spec.py

# Install the render worker and bundle the studio composition for it (Phase 9).
worker:
    pnpm --dir render-worker install --frozen-lockfile --silent
    pnpm --dir render-worker --silent bundle
