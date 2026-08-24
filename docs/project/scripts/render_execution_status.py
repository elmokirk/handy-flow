from pathlib import Path
import json,re
root=Path(__file__).resolve().parents[1]
cat=json.loads((root/'TICKET_CATALOG.json').read_text(encoding='utf-8'))['tickets']
run_dir=root/'status/runs'

def scalar(text,key,default=''):
    if not text.startswith('---\n'): return default
    end=text.find('\n---\n',4); fm=text[4:end] if end>=0 else ''
    m=re.search(rf'(?m)^{re.escape(key)}:\s*["\']?([^"\'\n]+)',fm)
    return m.group(1).strip() if m else default

rows=[]
for tid,t in sorted(cat.items(), key=lambda kv:(str(kv[1]['phase']),kv[0])):
    status=t.get('status','NOT_STARTED'); attempt='0'; branch=''
    rp=run_dir/f'{tid}.md'
    if rp.exists():
        txt=rp.read_text(encoding='utf-8'); status=scalar(txt,'run_status',status); attempt=scalar(txt,'attempt_count','0'); branch=scalar(txt,'branch','')
    rows.append((t['phase'],tid,t['gate'],status,attempt,branch))
out=['---','id: "execution-status"','title: "Execution Status"','type: "status"','status: "generated"','version: "1.1"','updated: "generated"','project: "custom-handy"','baseline_id: "handy-main-2026-08-24-af48dd68"','---','','# Execution Status','','> Generated from `TICKET_CATALOG.json` plus per-ticket RUN_STATE frontmatter. Do not edit manually.','','| Phase | Ticket | Gate | Status | Attempt | Branch |','|---|---|---|---|---:|---|']
out += [f'| {ph} | {tid} | {gate} | {st} | {att} | `{br}` |' for ph,tid,gate,st,att,br in rows]
(root/'status/EXECUTION_STATUS.md').write_text('\n'.join(out)+'\n',encoding='utf-8')
print(f'generated {len(rows)} ticket rows')
