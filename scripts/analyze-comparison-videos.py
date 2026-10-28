"""Output-only measurements for generated comparison clips. No model calls or code execution."""
import collections, hashlib, io, json, math, pathlib, struct, subprocess, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont
root=pathlib.Path(sys.argv[1] if len(sys.argv)>1 else '.clipkit/video-comparison-20261028').resolve()
tools=json.loads((root/'analysis-tools.json').read_text()); ffmpeg=tools['ffmpeg']; ffprobe=tools['ffprobe']
analysis=root/'analysis';analysis.mkdir(exist_ok=True)
spec=json.loads((root/'protocol.json').read_text())['comparison']; cache={};records=[]
def run(args,timeout=120): return subprocess.run(args,capture_output=True,timeout=timeout)
def ratio(value):
 try:
  a,b=value.split('/');return float(a)/float(b)
 except (ValueError,ZeroDivisionError,AttributeError):return None
def intervals(mask,fps):
 found=[];start=None
 for i,yes in enumerate(list(mask)+[False]):
  if yes and start is None:start=i
  elif not yes and start is not None:
   if (i-start)/fps>=.5:found.append({'start':round(start/fps,3),'end':round(i/fps,3),'duration':round((i-start)/fps,3)})
   start=None
 return found
def faststart(file):
 atoms=[]
 with file.open('rb') as f:
  offset=0;total=file.stat().st_size
  while offset+8<=total:
   f.seek(offset);header=f.read(8)
   if len(header)<8:break
   size,kind=struct.unpack('>I4s',header)
   if size==1:size=struct.unpack('>Q',f.read(8))[0]
   if size==0:size=total-offset
   if size<8:break
   atoms.append(kind.decode('ascii',errors='replace'));offset+=size
 return {'topLevelAtoms':atoms,'fastStart':atoms.index('moov')<atoms.index('mdat') if 'moov' in atoms and 'mdat' in atoms else None}
def finite(value):
 try:
  result=float(value);return result if math.isfinite(result) else None
 except (TypeError,ValueError):return None
def measure(file):
 digest=hashlib.sha256(file.read_bytes()).hexdigest()
 if digest in cache:return dict(cache[digest])
 folder=analysis/digest[:16];folder.mkdir(exist_ok=True)
 probe=run([ffprobe,'-v','error','-show_format','-show_streams','-of','json',str(file)])
 if probe.returncode:return {'sha256':digest,'error':'Probe failed','diagnostic':probe.stderr.decode(errors='replace')[-300:]}
 data=json.loads(probe.stdout);v=next((s for s in data['streams'] if s['codec_type']=='video'),None);audio=[s for s in data['streams'] if s['codec_type']=='audio'];fmt=data['format']
 if not v:return {'sha256':digest,'error':'No video stream'}
 fps=ratio(v.get('avg_frame_rate')) or ratio(v.get('r_frame_rate')) or 30;duration=float(fmt['duration']);size=file.stat().st_size
 decode=run([ffmpeg,'-v','error','-i',str(file),'-an','-vf','scale=160:90','-pix_fmt','rgb24','-f','rawvideo','pipe:1'])
 if decode.returncode:return {'sha256':digest,'error':'Full video decode failed','diagnostic':decode.stderr.decode(errors='replace')[-300:]}
 pixels=np.frombuffer(decode.stdout,dtype=np.uint8).reshape((-1,90,160,3));f=pixels.astype(np.float32)/255;delta=np.mean(np.abs(np.diff(f,axis=0)),axis=(1,2,3));luma=np.sum(f*np.array([.2126,.7152,.0722]),axis=3)
 freeze=intervals(delta<.0005,fps);black=(np.mean(luma,axis=(1,2))<.02)&(np.quantile(luma,.99,axis=(1,2))<.03)
 palette={}
 sample=pixels[::max(1,int(round(fps/2)))].reshape((-1,3)).astype(np.float32)
 for color in spec['palette']:
  rgb=np.array([int(color[i:i+2],16) for i in [1,3,5]]);palette[color]=round(float(np.mean(np.linalg.norm(sample-rgb,axis=1)<=16))*100,3)
 sample_times=[0,.5,1,2,3.5,5,7.5,9,10,12,13.5,14.8];indices=[min(len(pixels)-1,round(t*fps)) for t in sample_times]
 expression='+'.join('eq(n\\,%d)'%i for i in sorted(set(indices)))
 contact=run([ffmpeg,'-y','-v','error','-i',str(file),'-vf','select='+expression+',scale=480:270','-fps_mode','vfr','-threads','1',str(folder/'frame-%02d.png')])
 sheet_path=None
 if contact.returncode==0:
  thumbs=sorted(folder.glob('frame-*.png'));sheet=Image.new('RGB',(1920,3*300),'#0d1218');draw=ImageDraw.Draw(sheet)
  for i,thumb in enumerate(thumbs[:12]):
   x=(i%4)*480;y=(i//4)*300;sheet.paste(Image.open(thumb).convert('RGB'),(x,y+28));draw.text((x+10,y+7),f'{sample_times[i]:04.1f} s',fill='white')
  sheet.save(folder/'contact.png');sheet_path=str((folder/'contact.png').relative_to(root))
 peak=None;loudness=None
 if audio:
  sound=run([ffmpeg,'-hide_banner','-i',str(file),'-vn','-af','loudnorm=I=-16:TP=-1:LRA=11:print_format=json','-f','null','-'])
  import re
  match=re.search(r'\{\s*"input_i"[\s\S]*?\}',sound.stderr.decode(errors='replace'))
  if match:
   loud=json.loads(match.group());peak=finite(loud.get('input_tp'));loudness=finite(loud.get('input_i'))
 result={'sha256':digest,'bytes':size,'megabytes':round(size/1e6,4),'durationSeconds':duration,'width':v['width'],'height':v['height'],'aspect':round(v['width']/v['height'],6),'fps':fps,'nominalFps':ratio(v.get('r_frame_rate')),'decodedFrames':len(pixels),'codec':v['codec_name'],'profile':v.get('profile'),'pixelFormat':v.get('pix_fmt'),'sampleAspect':v.get('sample_aspect_ratio'),'videoBitrateKbps':round(float(v.get('bit_rate',0))/1000,3),'containerBitrateKbps':round(size*8/duration/1000,3),'encoder':fmt.get('tags',{}).get('encoder'),'colorSpace':v.get('color_space','unspecified'),'colorTransfer':v.get('color_transfer','unspecified'),'colorPrimaries':v.get('color_primaries','unspecified'),'decodeErrors':decode.stderr.decode(errors='replace').strip(),'audioStreams':len(audio),'integratedLUFS':loudness,'truePeakDBTP':peak,'motionMeanAbsoluteDeltaPercent':round(float(np.mean(delta))*100,5),'motionP95DeltaPercent':round(float(np.quantile(delta,.95))*100,5),'dynamicFrameFraction':round(float(np.mean(delta>=.0005)),4),'lowMotionHolds':freeze,'blackFrameCount':int(np.sum(black)),'blackIntervals':intervals(black,fps),'paletteMatchPercent':palette,'perFrameMotionPercent':[round(float(x)*100,5) for x in delta],'sampleTimes':sample_times,'contactSheet':sheet_path,'measurementNotes':['Motion is low-resolution mean absolute RGB difference, not a quality score.','Low-motion intervals may be intentional readable holds.','Palette percentages include background/decorative geometry; they are not OCR or text compliance.','Text bounds, exact glyphs and visual hierarchy require source/frame inspection.'],**faststart(file)}
 result['technicalChecks']={'duration':abs(duration-15)<=1/30+.01,'dimensions':v['width']==1920 and v['height']==1080,'fps':abs(fps-30)<.01,'frameCount':len(pixels)==450,'silentBrief':len(audio)==0,'fullDecode':not bool(result['decodeErrors'])}
 cache[digest]=dict(result);return result
for file in sorted(root.glob('rep-*/rounds/round-*/result.mp4')):
 run_name=file.parts[-4];round_index=int(file.parts[-2].split('-')[-1]);metrics=measure(file) if file.is_file() else {'error':'Checkpoint linked to a temporary export that was not retained; video unavailable for inspection','retentionFailure':True};records.append({'run':run_name,'round':round_index,'video':str(file.relative_to(root)),**metrics});print(run_name,round_index,metrics.get('bytes'),metrics.get('technicalChecks'),flush=True)
(root/'video-analysis.json').write_text(json.dumps({'method':'Output-only ffprobe/full decode, temporal RGB differences and fixed frame samples. No source-format bonus or automatic aesthetic winner.','videos':records},indent=2));print('Analyzed',len(records),'stage videos,',len(cache),'unique encoded clips',flush=True)
