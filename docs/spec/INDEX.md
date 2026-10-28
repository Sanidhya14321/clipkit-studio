# Spec index

The spec is `docs/MASTER_PROMPT.md`, split one file per section by `scripts/split-spec.py`.
Read this index, then only the files your phase needs. Never edit files in this folder.

| File | Summary | Phases |
| --- | --- | --- |
| [00-how-to-use.md](00-how-to-use.md) | Rules for the building agent: phase order, no extra features, verify versions, no invented data | all |
| [01-mission.md](01-mission.md) | The user experience end to end and the non-negotiables (non-destructive, turn-taking, speed) | all (once), 2, 3 |
| [02-architecture.md](02-architecture.md) | Components and language split, Rust crates to use, rendering strategy, distribution | 0, 1, 2, 9 |
| [03-layout-and-model.md](03-layout-and-model.md) | Repo layout, project data model (Project, Source, Track, Item, VoiceSlot, Comment, Brief), storage | 0, 1, 2 |
| [04-pipeline.md](04-pipeline.md) | Pipeline stages, server-enforced guardrails, quality gates | 1, 4 |
| [05-studio.md](05-studio.md) | Auto-launch, what the user sees while the agent works, turn-taking, idle editing | 2, 3, 6 |
| [06-agent-integration.md](06-agent-integration.md) | Shipped Claude Code plugin, MCP and CLI parity, other agents, voice lanes and event bridge | 2, 3, 7 |
| [07-assets.md](07-assets.md) | Knowledge base layout, asset cards, retrieval and adaptation ladders, starter content, Source Hub | 4, 8 |
| [08-features.md](08-features.md) | Brief Studio, footage editing, voice, comments and hover-select, direction engine, history and export | 4 (8.1, 8.5), 5 (8.2), 6 (8.4), 7 (8.3), 9 (8.6) |
| [09-token-efficiency.md](09-token-efficiency.md) | Runtime token principles, targets, token meter, the shipped runtime skill text | 3, 4, 8, 9 |
| [10-testing.md](10-testing.md) | Test layers, fixtures, with/without-plugin benchmark, debuggability, evidence rule | all; 0 (10.2), 9 (10.3) |
| [11-claude-code-setup.md](11-claude-code-setup.md) | Build-time Claude Code setup: CLAUDE.md, spec split, skills, subagents, hooks, permissions | 0 |
| [12-phases.md](12-phases.md) | Build phases 0 to 10, cut line, definition of done | all |
| [13-kickoff.md](13-kickoff.md) | Kickoff and per-phase messages to paste into a session | 0 |

## Phase to files

- Phase 0, setup: 00, 03 (3.1), 10 (10.2), 11, 12
- Phase 1, core: 00, 02 (2.2), 03, 04, 10 (10.1, 10.4)
- Phase 2, server and studio shell: 00, 01, 02, 03, 05, 06 (6.2)
- Phase 3, plugin and auto-launch: 00, 05 (5.1), 06, 09
- Phase 4, generate slice: 00, 04 (4.3), 07, 08 (8.1, 8.5), 09
- Phase 5, edit slice: 00, 08 (8.2), 10 (10.2)
- Phase 6, interaction slice: 00, 05, 08 (8.4)
- Phase 7, voice slice: 00, 06 (6.4), 08 (8.3)
- Phase 8, assets at scale: 00, 07, 09
- Phase 9, polish and proof: 00, 02 (2.3), 08 (8.6), 10
- Phase 10, stretch: 00, 12
