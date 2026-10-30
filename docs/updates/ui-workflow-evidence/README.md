# Workflow evidence

Final run: 2026-10-30, `codex/ui-pro`. Use the runtime versions recorded in `UI_WORKFLOW.md`. Session-specific environment variables are in `/tmp/ck-ui-test-env.sh`; no credentials are stored here.

Commands run from the UI worktree:

```sh
pnpm --dir studio build
cargo build -p ck-cli
pnpm --dir render-worker bundle
just check
just test
pnpm --dir studio exec playwright test 'phase.*\.spec\.ts|ui-workflows\.spec\.ts' --config playwright.config.ts --repeat-each=5 --workers=1 --output test-results/real
pnpm --dir studio exec playwright test --config playwright.ui.config.ts --repeat-each=5 --output test-results/ui
```

For local Chrome, set `CK_BROWSER_CHANNEL=chrome`; for the worker, set `CK_RENDER_WORKER` to the bundled worker folder and, when needed, `CK_RENDER_BROWSER` to the installed Chrome executable. Build ck first and point `CK_BIN` at it when using a nondefault Cargo target directory. Generate synthetic fixtures with `just fixtures` before the media/image tests. These commands use existing project dependencies.

The final actual browser runs wrote to `test-results-real/` and `test-results-ui/`; those artifacts were subsequently moved under ignored `studio/test-results/ui-workflow-final/`. Future commands above use ignored subdirectories directly. Prettier briefly picked up the temporary artifact directories during a concurrent check; the final `just check` was rerun after archiving them and passed.

`real-final-five.txt`: 26 real-server workflows × five runs = 130 passed. `shell-final-five.txt`: eight frontend-only contracts × five runs = 40 passed. `just-check.txt`: format, clippy, 377 Rust tests (one skipped), 41 hooks, TypeScript and 47 frontend unit tests. `just-test.txt`: full Rust/hook suite. `title-contrast-regression.txt` deliberately records the failure before the contrast fix; it is not the final result.

The 20 current layout screenshots use synthetic frontend-contract data. The three `real-*.png` screenshots use actual Rust-server projects and synthetic media. They are not owner-media or live-provider demonstrations. Read `ASSET_USAGE_AUDIT.md` for which asset paths are complete and which remain gaps.
