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

# Studio dev server (studio/ is built in Phase 2).
studio-dev:
    @echo "PLACEHOLDER: studio/ is created in Phase 2 (spec 12)"; exit 1

# Playwright browser tests (Phase 2 onward).
e2e:
    @echo "PLACEHOLDER: Playwright e2e suite arrives in Phase 2 (spec 10.1)"; exit 1

# Generate synthetic fixture media and ground truth into fixtures/generated/.
fixtures out="fixtures/generated":
    fixtures/generate.sh {{out}}

# With/without-plugin benchmark (Phase 9; costs tokens, run on demand).
bench:
    @echo "PLACEHOLDER: bench harness (A0, A1, B, ablations) arrives in Phase 9 (spec 10.3)"; exit 1

# Re-split docs/MASTER_PROMPT.md into docs/spec/ (only when the master prompt changes).
split-spec:
    python3 -I scripts/split-spec.py
