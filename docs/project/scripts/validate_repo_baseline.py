from pathlib import Path
import argparse,json,subprocess,sys
parser=argparse.ArgumentParser(); parser.add_argument('--repo',required=True); a=parser.parse_args()
repo=Path(a.repo).resolve(); kit=Path(__file__).resolve().parents[1]
lock=json.loads((kit/'BASELINE_LOCK.json').read_text(encoding='utf-8')); m=json.loads((kit/'REPO_PATH_MAP.json').read_text(encoding='utf-8'))
errors=[]
def out(*args): return subprocess.check_output(['git','-C',str(repo),*args],text=True).strip()
try: head=out('rev-parse','HEAD')
except Exception as e: errors.append(f'git HEAD unavailable: {e}'); head=None
if head:
    r=subprocess.run(['git','-C',str(repo),'merge-base','--is-ancestor',lock['baseline_commit'],head],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    if r.returncode: errors.append(f"pinned baseline {lock['baseline_commit']} is not an ancestor of HEAD {head}")
missing=[p for p in m['existing_required'] if not (repo/p).exists()]
collisions=[p for p in m['planned_new'] if (repo/p).exists()]
if missing: errors.append('missing existing_required: '+', '.join(missing))
if collisions: errors.append('planned_new collisions: '+', '.join(collisions))
result={'repo':str(repo),'head':head,'baseline':lock['baseline_commit'],'missing':missing,'planned_new_collisions':collisions,'errors':errors}
(kit/'status/REPO_PATH_VERIFICATION.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
print(json.dumps(result,indent=2)); sys.exit(1 if errors else 0)
