from pathlib import Path
import re, json, sys

root = Path(__file__).resolve().parents[1]
# Workspace layout: repo/, worktrees/ and node_modules live inside the kit root
# but belong to the implementation workspace, not to the control plane.
EXCLUDED_DIRS = {"repo", "worktrees", "node_modules", ".git"}
def _excluded(p: Path) -> bool:
    return any(part in EXCLUDED_DIRS for part in p.relative_to(root).parts[:-1])
mds = [p for p in root.rglob("*.md") if not _excluded(p)]
errors = []

# Frontmatter
for p in mds:
    txt = p.read_text(encoding="utf-8")
    if not txt.startswith("---\n"):
        errors.append(f"missing frontmatter: {p.relative_to(root)}")
        continue
    end = txt.find("\n---\n", 4)
    if end < 0:
        errors.append(f"unclosed frontmatter: {p.relative_to(root)}")
        continue
    fm = txt[4:end]
    for key in ["id:", "title:", "type:", "status:", "version:", "updated:", "project:", "baseline_id:"]:
        if key not in fm:
            errors.append(f"frontmatter missing {key} in {p.relative_to(root)}")

# Obsidian wiki link validation
targets = {}
for p in mds:
    rel = p.relative_to(root).as_posix()
    stem_path = rel[:-3] if rel.endswith(".md") else rel
    targets[stem_path] = p
    targets[p.stem] = p  # also allow unique simple names

wiki = re.compile(r"\[\[([^\]|#]+)(?:#[^\]|]+)?(?:\|[^\]]+)?\]\]")
for p in mds:
    txt = p.read_text(encoding="utf-8")
    for raw in wiki.findall(txt):
        target = raw.strip()
        if target not in targets:
            errors.append(f"broken wiki link in {p.relative_to(root)} -> {target}")

result = {"markdown_files": len(mds), "errors": errors}
(root/"VALIDATION.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
print(json.dumps(result, indent=2))
sys.exit(1 if errors else 0)
