from pathlib import Path
import json,subprocess,sys
root=Path(__file__).resolve().parents[1]
checks=[root/'scripts/validate_starter_kit.py',root/'scripts/validate_control_plane.py']
for c in checks:
    r=subprocess.run([sys.executable,str(c)],cwd=root)
    if r.returncode:
        print('Planning score invalid: structural validator failed.',file=sys.stderr); sys.exit(1)
score=json.loads((root/'PLANNING_SCORECARD.json').read_text(encoding='utf-8'))
print(json.dumps(score,indent=2))
