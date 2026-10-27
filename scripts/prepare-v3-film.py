"""Create original sound and crop genuine recorded editor evidence. No synthetic UI."""
import pathlib,json,subprocess,math,wave,struct,random
root=pathlib.Path.cwd();out=root/'artifacts/film-0.3';out.mkdir(parents=True,exist_ok=True);ff=str(root/'node_modules/ffmpeg-static/ffmpeg')
workflow=json.load(open(root/'.clipkit/v3-verification/workflow.json'));human=json.load(open(root/'.clipkit/v3-verification/human-edit.json'));full=next((root/'artifacts/workflow-0.3').glob('*.webm'));tail=next((root/'artifacts/workflow-human-0.3').glob('*.webm'))
def run(args):subprocess.run([ff,'-v','error','-y',*args],check=True)
filter_range='[0:v]split=2[a][b];[a]crop=270:40:0:695,scale=900:134[top];[b]crop=300:230:1066:410,scale=720:552[bottom];color=c=0x1c1c1c:s=920x720:r=30:d=6[canvas];[canvas][top]overlay=10:10[tmp];[tmp][bottom]overlay=100:158:shortest=1,format=yuv420p[out]'
run(['-ss',str(workflow['proofStart']),'-i',str(full),'-filter_complex',filter_range,'-map','[out]','-t','6','-r','30','-an','-c:v','libx264','-crf','16',str(out/'range-proof.mp4')])
filter_human='[0:v]split=2[a][b];[a]crop=250:446:537:84,scale=400:714[left];[b]crop=300:380:1066:280,scale=450:570[right];color=c=0x1c1c1c:s=920x720:r=30:d=5[canvas];[canvas][left]overlay=0:3[tmp];[tmp][right]overlay=460:75:shortest=1,format=yuv420p[out]'
run(['-ss',str(max(0,human['events'][0]['seconds']-.8)),'-i',str(tail),'-filter_complex',filter_human,'-map','[out]','-t','5','-r','30','-an','-c:v','libx264','-crf','16',str(out/'human-proof.mp4')])
# Original synthesized score: selective impacts, bass pulses, noise hats; quiet proof hold.
sr=48000;rng=random.Random(301);data=bytearray();notes=[110,138.59,164.81,123.47];impacts=[0,3,5,10,16,21,24]
for n in range(25*sr):
 t=n/sr;beat=t%.5;section=.24 if 10<=t<16 else 1;attack=math.exp(-beat*22);kick=math.sin(2*math.pi*(52*beat+2*(1-math.exp(-beat*23))))*attack*.23
 note=notes[int(t/2)%len(notes)];pluck=math.sin(2*math.pi*note*t)*math.exp(-beat*13)*.07
 hatphase=t%(.125 if 5<=t<10 else .25);hat=(rng.random()*2-1)*math.exp(-hatphase*130)*.025
 impact=sum((rng.random()*2-1)*math.exp(-(t-a)*23)*.11 for a in impacts if 0<=t-a<.3)
 v=(kick+pluck+hat)*section+impact;v*=min(1,t/.015,(25-t)/.2);sample=int(max(-.98,min(.98,v))*32767);data.extend(struct.pack('<hh',sample,sample))
with wave.open(str(out/'original-rhythm.wav'),'wb') as w:w.setnchannels(2);w.setsampwidth(2);w.setframerate(sr);w.writeframes(data)
run(['-i',str(out/'original-rhythm.wav'),'-af','loudnorm=I=-16:TP=-1.5:LRA=9','-ar','48000',str(out/'original-rhythm-normalized.wav')])
run(['-i',str(full),'-i',str(tail),'-filter_complex','[0:v]fps=30,format=yuv420p[a];[1:v]fps=30,format=yuv420p[b];[a][b]concat=n=2:v=1:a=0[v]','-map','[v]','-an','-c:v','libx264','-crf','23',str(root/'artifacts/clipkit-workflow-0.3.0.mp4')])
(out/'README.txt').write_text('Range and control proof clips are crops of actual localhost recordings; not fabricated UI. Original rhythm is newly synthesized for this demo. Full unaltered interaction timing is retained in clipkit-workflow-0.3.0.mp4.\n')
print('Prepared genuine editor crops, original sound, and full workflow recording')
