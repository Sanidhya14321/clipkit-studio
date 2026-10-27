"""Pattern scan tracked Git history before publishing; never prints secret values.

This is a targeted scan and file-exclusion check, not a guarantee that arbitrary
credentials or personal information can be detected automatically.
"""
import json
import hashlib
import re
import subprocess
from pathlib import Path


def git(*args, data=None):
    return subprocess.check_output(["git", *args], input=data)


objects = git("rev-list", "--objects", "--all").decode().splitlines()
paths = {}
for entry in objects:
    sha, _, name = entry.partition(" ")
    paths.setdefault(sha, name)
# Include the index so unpublished additions/edits are scanned before commit.
for entry in git("ls-files", "-s").decode().splitlines():
    metadata, name = entry.split("\t", 1)
    sha = metadata.split()[1]
    paths.setdefault(sha, name)
types = git("cat-file", "--batch-check=%(objectname) %(objecttype)",
            data=("\n".join(paths) + "\n").encode()).decode().splitlines()
patterns = {
    "GitHub credential": re.compile(r"(?:gh[pousr]_[A-Za-z0-9]{30,255}|github_pat_[A-Za-z0-9_]{40,255})"),
    "OpenAI credential": re.compile(r"sk-(?:proj-)?[A-Za-z0-9_-]{24,}"),
    "AWS access key": re.compile(r"AKIA[0-9A-Z]{16}"),
    "Private key": re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    "Quoted provider key": re.compile(r"(?i)(?:FISH_API_KEY|OPENAI_API_KEY|ANTHROPIC_API_KEY|GH_TOKEN|GITHUB_TOKEN)\s*[:=]\s*[\"']([^\"'\r\n]{12,})[\"']"),
}
findings = []
scanned = 0
for entry in types:
    sha, kind = entry.split()
    if kind != "blob":
        continue
    content = git("cat-file", "-p", sha)
    if b"\0" in content[:8000]:
        continue
    text = content.decode("utf-8", errors="replace")
    scanned += 1
    for label, pattern in patterns.items():
        if pattern.search(text):
            findings.append({"object": sha[:12], "path": paths[sha], "kind": label})

tracked = git("ls-files").decode().splitlines()
excluded = [".env", ".clipkit/project.json", "artifacts/example.tgz",
            "node_modules/example", "test-results/example", "playwright-report/example"]
ignore_results = {
    name: subprocess.run(["git", "check-ignore", "-q", name]).returncode == 0
    for name in excluded
}
# Only the selected, licensed starter footage may be committed as media.
# Project footage and arbitrary exports remain excluded.
redistributable_media = set()
manifest_path = Path("public/assets/manifest.json")
if manifest_path.exists():
    for asset in json.loads(manifest_path.read_text()).get("assets", []):
        candidate = Path("public") / asset.get("file", "")
        if (candidate.as_posix().startswith("public/assets/video/")
                and ".." not in candidate.parts
                and asset.get("license") in {"CC0-1.0", "CC-BY-4.0"}
                and candidate.is_file()
                and hashlib.sha256(candidate.read_bytes()).hexdigest() == asset.get("sha256")):
            redistributable_media.add(candidate.as_posix())
unsafe_files = [name for name in tracked if name.startswith((".clipkit/", "node_modules/", "artifacts/", "test-results/"))
                or (Path(name).name == ".env")
                or (name.lower().endswith((".docx", ".mp4", ".webm", ".tgz")) and name not in redistributable_media)]
report = {"sourceCommit": git("rev-parse", "HEAD").decode().strip(),
          "textBlobsScanned": scanned, "licensedStarterVideos": sorted(redistributable_media), "credentialPatternFindings": findings,
          "excludedPathsIgnored": ignore_results, "unexpectedTrackedPrivateOrBuildFiles": unsafe_files,
          "passed": not findings and not unsafe_files and all(ignore_results.values())}
destination = Path(".clipkit/verification/prd-audit/publication-scan.json")
destination.parent.mkdir(parents=True, exist_ok=True)
destination.write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
raise SystemExit(0 if report["passed"] else 1)
