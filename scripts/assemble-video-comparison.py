"""Present measured A/B outputs at original speed; retain the first weak pipeline result."""
from pathlib import Path
import subprocess,json,hashlib
from PIL import Image,ImageDraw,ImageFont
root=Path.cwd();out=root/'artifacts/ab-15s-0.4';ff=root/'node_modules/ffmpeg-static/ffmpeg'
def run(args):return subprocess.check_output([str(ff),'-v','error','-y',*args])
header=Image.new('RGB',(1080,120),'#18191c');d=ImageDraw.Draw(header)
font=ImageFont.truetype(str(root/'public/vendor/inter-latin.woff2'),24) if False else ImageFont.truetype(str(root/'public/vendor/space-grotesk.ttf'),26)
small=ImageFont.truetype(str(root/'public/vendor/space-grotesk.ttf'),17)
d.text((24,22),'PLAIN AGENT',font=font,fill='white');d.text((564,22),'VIDEO PIPELINE · V2',font=font,fill='#ff795f');d.text((24,64),'Same creative prompt · 15s · 30 fps',font=small,fill='#aaa');d.text((564,64),'Story → motion → render → critique',font=small,fill='#aaa');header.save(out/'comparison-header.png')
run(['-i',str(out/'without-plugin.mp4'),'-i',str(out/'with-plugin.mp4'),'-i',str(out/'comparison-header.png'),'-filter_complex','[0:v]scale=540:960,setsar=1[a];[1:v]scale=540:960,setsar=1[b];[a][b]hstack=inputs=2,pad=1080:1080:0:120:color=0x18191c[v];[v][2:v]overlay=0:0[out]','-map','[out]','-map','0:a','-t','15','-r','30','-c:v','libx264','-crf','18','-c:a','aac','-movflags','+faststart',str(out/'comparison-side-by-side.mp4')])
# Contact sheets sample the encoded outputs, not a reconstructed UI or composition.
for key in ['without','with']:
 run(['-i',str(out/(key+'-plugin.mp4')),'-vf','fps=1,scale=216:384,tile=5x3','-frames:v','1',str(out/(key+'-contact-sheet.png'))])
report=json.loads((out/'comparison.json').read_text());first=json.loads((out/'comparison-v1.json').read_text());assert report['promptSha256']==first['promptSha256'];assert report['with']['session']['prompt']==report['userPrompt']
for key in ['without','with']:
 report[key]['decodedAudioSha256']=run(['-i',str(out/(key+'-plugin.mp4')),'-map','0:a:0','-c:a','pcm_s16le','-f','hash','-hash','sha256','-']).decode().strip()
report['with']['usageCoverage']={'complete':all('actualUsage' in m for m in report['with']['session']['metrics']),'missingStages':[m['stage'] for m in report['with']['session']['metrics'] if 'actualUsage' not in m]};report['with']['usageTotalsNote']='Reported-stage subtotal only when coverage is incomplete.'
report['decodedAudioIdentical']=report['without']['decodedAudioSha256']==report['with']['decodedAudioSha256'];report['creativePromptByteIdentical']=True
report['with']['totalAgentSeconds']=sum(m['elapsedMs'] for m in report['with']['session'].get('metrics',[]))/1000
report['with']['usageTotals']={k:sum(m.get('actualUsage',{}).get(k,0) for m in report['with']['session'].get('metrics',[])) for k in ['input','output','cached']}
report['editorialAssessment']={'baseline':'Richer graphic density, bolder type hierarchy and more micro-detail; also uses recognizable poster-like scenes.','initialPipeline':'Too restrictive and poster-like; critic requested changes and still left a transition-edge defect. Preserved as with-plugin-v1.mp4, not hidden.','revisedPipeline':'Stronger shared-object continuity: a ribbon guides a K through a pose-selection band into the closing identity. The critic required a repair to make the selection meaningful and keep the closing signature clear.','conclusion':'The revised pipeline gives better structured continuity and an actual quality-control loop. It does not establish universal aesthetic superiority over the plain agent; the plain version remains competitive and is faster.','limits':'One creative brief, no blinded panel or audience test. Stills critique is not complete moving/audio review. Both original-speed videos are provided. Workflow, instructions and motion kit differ intentionally.'}
(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n');(out/'SHA256SUMS').write_text(''.join(hashlib.sha256((out/f).read_bytes()).hexdigest()+'  '+f+'\n' for f in ['PROMPT.txt','without-plugin.mp4','with-plugin.mp4','with-plugin-v1.mp4','comparison-side-by-side.mp4','comparison.json']))
print(json.dumps({'samePrompt':True,'decodedAudioIdentical':report['decodedAudioIdentical'],'plainAgentSeconds':report['without']['elapsedMs']/1000,'pipelineAgentSeconds':report['with']['totalAgentSeconds'],'pipelineStages':[m['stage'] for m in report['with']['session']['metrics']]},indent=2))
