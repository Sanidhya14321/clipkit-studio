"""Recover authored source through literal string edits; never executes recorded code."""
import ast, hashlib, json, pathlib, re, shlex, sys
root=pathlib.Path(sys.argv[1] if len(sys.argv)>1 else '.clipkit/video-comparison-20261028')
receipts=[]
for run in root.glob('rep-*-A0'):
    source=None; versions={}; edits=[]
    for events in (run/'.clipkit/trace').glob('*/events.jsonl'):
        for line in events.read_text().splitlines():
            event=json.loads(line); payload=event.get('payload',{}); item=payload.get('item',{})
            if payload.get('type')!='item.completed' or item.get('type')!='command_execution' or item.get('exit_code')!=0: continue
            try: shell=shlex.split(item['command'])[-1]
            except (KeyError,ValueError): continue
            initial=re.search(r"cat\s*>\s*(?:/work/)?render\.py\s*<<['\"]?(\w+)['\"]?\n(.*?)\n\1(?:\n|$)",shell,re.S)
            round=event.get('round',0)
            if initial:
                source=initial.group(2)+'\n';versions[round]=source;edits.append({'round':round,'kind':'literal here-document','commandSHA256':hashlib.sha256(item['command'].encode()).hexdigest()});continue
            if source is None: continue
            patch=re.search(r"python3\s+-\s*<<['\"]?(\w+)['\"]?\n(.*?)\n\1(?:\n|$)",shell,re.S)
            if not patch or 'write_text' not in patch.group(2):continue
            env={}; updated=False
            def value(node):
                if isinstance(node,ast.Constant):return node.value
                if isinstance(node,ast.Name):return env[node.id]
                if isinstance(node,ast.BinOp) and isinstance(node.op,ast.Add):return value(node.left)+value(node.right)
                if isinstance(node,ast.Call):
                    if isinstance(node.func,ast.Name) and node.func.id=='Path':
                        target=value(node.args[0])
                        if target not in ['render.py','/work/render.py']:raise ValueError('Unexpected path')
                        return ('virtual-path',target)
                    if isinstance(node.func,ast.Attribute):
                        obj=value(node.func.value)
                        if node.func.attr=='read_text' and isinstance(obj,tuple):return source
                        if node.func.attr=='replace' and isinstance(obj,str):return obj.replace(*[value(arg) for arg in node.args])
                raise ValueError('Non-literal edit expression')
            for node in ast.parse(patch.group(2)).body:
                try:
                    if isinstance(node,ast.Assign):
                        result=value(node.value)
                        for target in node.targets:
                            if isinstance(target,ast.Name):env[target.id]=result
                    elif isinstance(node,ast.Expr) and isinstance(node.value,ast.Call) and isinstance(node.value.func,ast.Attribute) and node.value.func.attr=='write_text':
                        if not isinstance(value(node.value.func.value),tuple):raise ValueError('Unexpected write target')
                        result=value(node.value.args[0])
                        if not isinstance(result,str):raise ValueError('Non-text replacement')
                        source=result;updated=True
                except (ValueError,KeyError,TypeError):continue
            if updated:versions[round]=source;edits.append({'round':round,'kind':'literal string replacement','commandSHA256':hashlib.sha256(item['command'].encode()).hexdigest()})
    for round,source in versions.items():
        folder=run/'rounds'/('round-'+str(round))/'recovered-source';folder.mkdir(parents=True,exist_ok=True);(folder/'render.py').write_text(source)
    if versions:receipts.append({'run':run.name,'versionsRecovered':sorted(versions),'edits':edits,'method':'Parsed shell here-documents and interpreted only literal read_text/str.replace/write_text expressions on a virtual file. No recorded source or shell command executed.'})
(root/'recovered-source-provenance.json').write_text(json.dumps(receipts,indent=2));print(json.dumps(receipts,indent=2))
