# Benchmark (spec 10.3)

`scenarios.json` lists the arms (A0, A1, B and the C ablations) and the 15 scenarios. `run.sh`
does one run in a fresh workspace with the agent given in `AGENT_CMD` and appends a line to
`results.jsonl`; `ck bench report bench/results.jsonl` gives medians, ranges and pass rates per
arm and scenario. Tokens come only from the agent's own usage report; when an arm reports none,
the report says not measured.

Results: PLACEHOLDER. No run has been made yet: it needs real agents, the owner-supplied media
marked PLACEHOLDER in `scenarios.json`, and tokens. Fairness rules (same model, agent version,
machine, prompts and limits; at least 3 runs per arm) are the operator's to keep; a clean
baseline for A0 and A1 is a fresh workspace without `ck init`.
