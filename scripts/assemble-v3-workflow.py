"""Join real localhost recordings, preserving the waiting and interaction time."""
from pathlib import Path
import subprocess,json
root=Path.cwd();ff=root/'node_modules/ffmpeg-static/ffmpeg'
files=[next((root/'artifacts/workflow-0.3').glob('*.webm')),next((root/'artifacts/workflow-human-0.3').glob('*.webm'))]
exports=sorted((root/'artifacts/workflow-export-0.3').glob('*.webm'),key=lambda p:p.stat().st_mtime)
files += [exports[0],exports[-1]]
args=[str(ff),'-v','error','-y']
for f in files:args+=['-i',str(f)]
filters=';'.join(f'[{i}:v]fps=30,format=yuv420p[v{i}]' for i in range(len(files)))+';'+''.join(f'[v{i}]' for i in range(len(files)))+f'concat=n={len(files)}:v=1:a=0[v]'
subprocess.run(args+['-filter_complex',filters,'-map','[v]','-an','-c:v','libx264','-crf','23',str(root/'artifacts/clipkit-workflow-0.3.0.mp4')],check=True)
(root/'artifacts/clipkit-workflow-0.3.0.provenance.json').write_text(json.dumps({'segments':[str(f.relative_to(root)) for f in files],'treatment':'Concatenated actual captures at original interaction speed. Separate sessions for generation/range feedback, human editing, UI export and completed download. No fabricated UI or agent progress.','audio':'Silent desktop recording; the finished product reel has original sound.'},indent=2))
