import {ReelPlanSchema,allItems,type Project} from './index';
/** Compact, bounded planning context; scene source is loaded separately on demand. */
export function readReel(project:Project,args:{chapterId?:string;startFrame?:number;endFrame?:number;offset?:number;limit?:number}={}) {
 const plan=project.reel;if(!plan)return {planned:false as const,fps:project.settings.fps,chapters:undefined,next:null,score:undefined};
 const chapters=plan.chapters??[],offset=Math.max(0,args.offset??0),limit=Math.max(1,Math.min(20,args.limit??10));
 const chapter=args.chapterId?chapters.find(c=>c.id===args.chapterId):undefined;
 if(args.chapterId&&!chapter)throw Error('Unknown chapter');
 const start=chapter?.startFrame??args.startFrame,end=chapter?.endFrame??args.endFrame;
 if(start!==undefined&&(!Number.isInteger(start)||end===undefined||!Number.isInteger(end)||start<0||end<=start||end>plan.durationFrames))throw Error('Invalid chapter/range boundaries');
 const selected=start!==undefined&&end!==undefined;
 return {planned:true,fps:project.settings.fps,durationFrames:plan.durationFrames,pacing:plan.pacing,direction:plan.direction,chapter,chapters:selected?undefined:chapters.slice(offset,offset+limit),next:!selected&&offset+limit<chapters.length?offset+limit:null,...selected?{score:plan.score.filter(e=>e.startFrame<end!&&e.endFrame>start!),layers:allItems(project).filter(i=>Math.round(i.at*project.settings.fps)<end!&&Math.round((i.at+i.dur)*project.settings.fps)>start!).map(i=>({id:i.id,type:i.type,at:i.at,dur:i.dur,sceneId:i.sceneId,sceneVersion:i.sceneVersion,sourceId:i.src}))}:{}};
}
export function replaceChapterScore(project:Project,chapterId:string,score:unknown){
 const plan=project.reel,chapter=plan?.chapters?.find(c=>c.id===chapterId);if(!plan||!chapter)throw Error('Unknown chapter');
 const parsed=ReelPlanSchema.parse({...plan,score}).score;
 if(parsed.some(e=>e.startFrame<chapter.startFrame||e.endFrame>chapter.endFrame))throw Error('Score events must remain inside the selected chapter');
 const inside=(e:{startFrame:number;endFrame:number})=>e.startFrame>=chapter.startFrame&&e.endFrame<=chapter.endFrame;
 for(const event of plan.score.filter(e=>inside(e)&&e.protected)){const next=parsed.find(e=>e.id===event.id);if(!next||next.startFrame!==event.startFrame||next.endFrame!==event.endFrame)throw Error('Protected reading windows must retain exact timing');}
 const retained=plan.score.filter(e=>!inside(e));if(parsed.some(e=>retained.some(old=>old.id===e.id)))throw Error('Score IDs must be unique across chapters');
 project.reel=ReelPlanSchema.parse({...plan,score:[...retained,...parsed].sort((a,b)=>a.startFrame-b.startFrame)});
}
