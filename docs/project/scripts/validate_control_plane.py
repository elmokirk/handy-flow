from pathlib import Path
import json,sys,re
root=Path(__file__).resolve().parents[1]
cat=json.loads((root/'TICKET_CATALOG.json').read_text(encoding='utf-8'))
fm=json.loads((root/'FEATURE_MATRIX.json').read_text(encoding='utf-8'))
tickets=cat['tickets']; errors=[]

def markdown_deps(tid):
    p=root/'planning/tickets'/f'{tid}.md'
    if not p.exists(): return None
    text=p.read_text(encoding='utf-8')
    m=re.search(r'\*\*Dependencies:\*\*(.*)',text)
    return sorted(set(re.findall(r'planning/tickets/([A-Z0-9-]+)',m.group(1)))) if m else []

def frontmatter_scalar(p,key):
    text=p.read_text(encoding='utf-8')
    if not text.startswith('---\n'): return None
    end=text.find('\n---\n',4); fmtext=text[4:end] if end>=0 else ''
    m=re.search(rf'(?m)^{re.escape(key)}:\s*["\']?([^"\'\n]+)',fmtext)
    return m.group(1).strip() if m else None

for tid,t in tickets.items():
    tp=root/'planning/tickets'/f'{tid}.md'
    if not tp.exists(): errors.append(f'{tid}: markdown ticket missing'); continue
    if not t.get('allowed_paths'): errors.append(f'{tid}: empty allowed_paths')
    if not t.get('gate'): errors.append(f'{tid}: missing gate')
    if not t.get('qa_profile'): errors.append(f'{tid}: missing qa_profile')
    if t.get('max_agent_cycles',0)<=0: errors.append(f'{tid}: invalid max_agent_cycles')
    for dep in t.get('dependencies',[]):
        if dep not in tickets: errors.append(f'{tid}: missing dependency {dep}')
    md=markdown_deps(tid)
    if md is not None and sorted(t.get('dependencies',[]))!=md:
        errors.append(f'{tid}: markdown/catalog dependencies differ: md={md} catalog={sorted(t.get("dependencies",[]))}')
    if frontmatter_scalar(tp,'ticket_id')!=tid: errors.append(f'{tid}: frontmatter ticket_id mismatch')
    if frontmatter_scalar(tp,'gate')!=str(t.get('gate')): errors.append(f'{tid}: frontmatter gate mismatch')
    if frontmatter_scalar(tp,'catalog_ref')!=f'TICKET_CATALOG.json#{tid}': errors.append(f'{tid}: catalog_ref mismatch')
    overlap=set(t.get('allowed_paths',[])) & set(t.get('forbidden_paths',[]))
    if overlap: errors.append(f'{tid}: allowed/forbidden exact overlap {sorted(overlap)}')
    for ref in t.get('architecture_refs',[]):
        if not (root/ref).exists(): errors.append(f'{tid}: missing architecture ref {ref}')

state={}
def visit(n,stack):
    if state.get(n)==1: errors.append('cycle: '+' -> '.join(stack+[n])); return
    if state.get(n)==2: return
    state[n]=1
    for d in tickets[n].get('dependencies',[]): visit(d,stack+[n])
    state[n]=2
for n in tickets: visit(n,[])

for feat in fm['features']:
    if not feat.get('tickets'): errors.append(f"feature {feat.get('feature')}: no tickets")
    for tid in feat['tickets']:
        if tid not in tickets: errors.append(f"feature {feat['feature']}: missing ticket {tid}")
if cat.get('canonical_base_branch')!='custom/main': errors.append('canonical base branch must be custom/main')

# REST OpenAPI hard invariant without third-party YAML dependency.
openapi=(root/'orchestration/REST_OPENAPI_CONTRACT.yaml').read_text(encoding='utf-8')
for method in ['post:','put:','patch:','delete:']:
    if re.search(rf'(?m)^\s+{method}',openapi): errors.append(f'REST write method forbidden: {method[:-1].upper()}')
if 'bind: 127.0.0.1 only' not in openapi: errors.append('REST bind contract must be loopback-only')

mcp=json.loads((root/'orchestration/MCP_TOOL_CONTRACT.json').read_text(encoding='utf-8'))
if not mcp.get('read_only'): errors.append('MCP must be read_only')

# Hardening findings must be closed for P0/P1.
hard=json.loads((root/'HARDENING_FINDINGS.json').read_text(encoding='utf-8'))
for bucket in ['p0','p1']:
    for finding in hard.get(bucket,[]):
        if finding.get('status')!='RESOLVED': errors.append(f"{finding.get('id')}: hardening status {finding.get('status')}")

repo_map=json.loads((root/'REPO_PATH_MAP.json').read_text(encoding='utf-8'))
if repo_map.get('baseline_id')!=cat.get('baseline_id'): errors.append('REPO_PATH_MAP baseline_id mismatch')
if not repo_map.get('existing_required'): errors.append('REPO_PATH_MAP existing_required empty')
if not repo_map.get('planned_new'): errors.append('REPO_PATH_MAP planned_new empty')

if 'dto_contract: QUERY_DTO_CONTRACT.json' not in openapi: errors.append('REST OpenAPI must reference shared DTO contract')
if mcp.get('dto_contract')!='QUERY_DTO_CONTRACT.json': errors.append('MCP must reference shared DTO contract')
for word in ['create','update','delete','purge']:
    if any(name.startswith(word) for name in mcp.get('tools',{})): errors.append(f'MCP write-like tool present: {word}')

result={'tickets':len(tickets),'features':len(fm['features']),'errors':errors,'canonical_base_branch':cat.get('canonical_base_branch')}
(root/'CONTROL_PLANE_VALIDATION.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
print(json.dumps(result,indent=2)); sys.exit(1 if errors else 0)
