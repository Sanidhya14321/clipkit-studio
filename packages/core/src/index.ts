import {Update2Fields,FieldIdSchema,TokenSchema} from './brief';
import {PipelineFields,LayerPassSchema} from './pipeline-schema';
import {StoryboardSchema} from './storyboard';
import {DirectionPlanSchema} from './video-plan';
import {z} from 'zod';

export const id = (prefix = 'id') => `${prefix}_${globalThis.crypto.randomUUID().slice(0, 8)}`;
export const WordSchema = z.object({text: z.string(), start: z.number().nonnegative(), end: z.number().nonnegative()}).refine(w => w.end >= w.start, 'Word ends before it starts');
export type Word = z.infer<typeof WordSchema>;
export const SourceSchema = z.object({id: z.string(), kind: z.enum(['video','audio','image']), path: z.string(), proxyPath: z.string().optional(), audioPath: z.string().optional(), transcriptPath: z.string().optional(), name: z.string(), durationSec: z.number().nonnegative(), width: z.number().optional(), height: z.number().optional(), provenance: z.string().optional()});
export type Source = z.infer<typeof SourceSchema>;
export const ItemSchema = z.object({
  tokens:TokenSchema.optional(),rev:z.number().int().nonnegative().optional(),briefDeps:z.array(FieldIdSchema).max(29).optional(),tokenBindings:z.record(z.string(),z.string().max(100)).optional(),layout:z.object({role:z.string(),x:z.number(),y:z.number(),width:z.number(),height:z.number(),fontSize:z.number().optional()}).optional(),name:z.string().max(200).optional(),purpose:z.string().max(500).optional(),pass:LayerPassSchema.optional(),assetRef:z.string().optional(),z:z.number().int().optional(),
  id: z.string(), type: z.enum(['footage','clip','audio','voice','caption']),
  at: z.number().nonnegative(), dur: z.number().positive().max(36000),
  props: z.record(z.string(), z.unknown()).default({}),
  clipId: z.string().optional(),sceneId:z.string().optional(),scopeOf:z.string().optional(),sceneVersion:z.string().optional(),phaseOffset:z.number().nonnegative().optional(),phaseDuration:z.number().positive().optional(), src: z.string().optional(), srcIn: z.number().nonnegative().default(0), srcOut: z.number().nonnegative().optional(),
  volume: z.number().min(0).max(2).default(1), slotId: z.string().optional(), linkedTo: z.string().optional(), words: z.array(WordSchema).optional(),
}).superRefine((v,c) => {
  if (v.type === 'clip' && !v.clipId) c.addIssue({code:'custom',message:'Clip needs clipId'});
  if (['footage','audio'].includes(v.type) && !v.src) c.addIssue({code:'custom',message:'Media item needs src'});
  if (v.type === 'voice' && !v.slotId) c.addIssue({code:'custom',message:'Voice item needs slotId'});
});
export type Item = z.infer<typeof ItemSchema>;
export const TrackSchema = z.object({id:z.string(), name:z.string(), kind:z.enum(['video','overlay','audio','caption','voice']), items:z.array(ItemSchema)});
export type Track = z.infer<typeof TrackSchema>;
export const VoiceModeSchema = z.enum(['ai','recorded','cloned']);
export type VoiceMode = z.infer<typeof VoiceModeSchema>;
export const TakeSchema = z.object({id:z.string(), sourceId:z.string(), mode:VoiceModeSchema, durationSec:z.number().positive(), hash:z.string().optional(), words:z.array(WordSchema).optional(), createdAt:z.string()});
export type Take = z.infer<typeof TakeSchema>;
export const VoiceSlotSchema = z.object({id:z.string(), text:z.string().max(10000), mode:VoiceModeSchema, voiceRef:z.string().optional(), takes:z.array(TakeSchema), activeTakeByMode:z.object({ai:z.string().optional(),recorded:z.string().optional(),cloned:z.string().optional()}), targetWindow:z.object({start:z.number().nonnegative(),end:z.number().positive()}), status:z.enum(['empty','ready','stale'])});
export type VoiceSlot = z.infer<typeof VoiceSlotSchema>;
export const AnchorSchema = z.object({tStart:z.number().nonnegative(), tEnd:z.number().nonnegative().optional(),layerId:z.string().optional(),propPath:z.string().optional(),elementId:z.string().optional(),point:z.object({x:z.number(),y:z.number()}).optional()});
export type Anchor = z.infer<typeof AnchorSchema>;
export const CommentSchema = z.object({id:z.string(),author:z.enum(['human','agent','voice']),anchor:AnchorSchema,text:z.string().min(1).max(4000),status:z.enum(['open','working','resolved']),createdRev:z.number().int(),resolvedRev:z.number().int().optional(),replies:z.array(z.object({author:z.enum(['human','agent']),text:z.string()})),diff:z.array(z.object({path:z.string(),op:z.string(),before:z.unknown().optional(),after:z.unknown().optional(),beforeExists:z.boolean().optional(),afterExists:z.boolean().optional()}).passthrough()).optional()});
export type Comment = z.infer<typeof CommentSchema>;
export const ScoreEventSchema=z.object({id:z.string(),kind:z.enum(['shot','reading','transition','accent','audio']),label:z.string(),startFrame:z.number().int().nonnegative(),endFrame:z.number().int().positive(),mechanism:z.string().optional(),protected:z.boolean().optional()}).refine(e=>e.endFrame>e.startFrame,'Event must have a positive frame range');
export const ChapterSchema=z.object({id:z.string(),title:z.string(),summary:z.string().max(1500).optional(),startFrame:z.number().int().nonnegative(),endFrame:z.number().int().positive()}).refine(c=>c.endFrame>c.startFrame,'Chapter must have a positive range');
export const ReelPlanSchema=z.object({chapters:z.array(ChapterSchema).max(120).optional(),durationFrames:z.number().int().min(1).max(108000),pacing:z.enum(['fast','balanced','slow']),direction:z.string().max(4000),score:z.array(ScoreEventSchema).max(2000)}).refine(p=>p.score.every(e=>e.endFrame<=p.durationFrames)&&(p.chapters??[]).every(c=>c.endFrame<=p.durationFrames),'Score or chapter exceeds reel duration').refine(p=>new Set(p.chapters?.map(c=>c.id)).size===(p.chapters?.length??0)&&(p.chapters??[]).every((c,i,all)=>!all.some((other,j)=>j!==i&&c.startFrame<other.endFrame&&c.endFrame>other.startFrame)),'Chapters need unique IDs and nonoverlapping ranges');
export const ProjectSchema = z.object({...Update2Fields,...PipelineFields,videoPlans:z.record(z.string(),StoryboardSchema).optional(),directionPlans:z.record(z.string(),DirectionPlanSchema).optional(),reel:ReelPlanSchema.optional(),schemaVersion:z.literal(1),id:z.string(),name:z.string().min(1).max(200),rev:z.number().int().nonnegative(),settings:z.object({fps:z.number().int().min(1).max(60),width:z.number().int().positive().max(3840),height:z.number().int().positive().max(3840)}),sources:z.record(z.string(),SourceSchema),tracks:z.array(TrackSchema),voiceSlots:z.array(VoiceSlotSchema),comments:z.array(CommentSchema),brand:z.object({colors:z.array(z.string()),fonts:z.array(z.string())}).optional()}).superRefine((p,c)=>{
  if(p.reel&&p.tracks.some(t=>t.items.some(i=>Math.round((i.at+i.dur)*p.settings.fps)>p.reel!.durationFrames)))c.addIssue({code:'custom',message:'Timeline content exceeds the reel frame budget'});
  const ids = new Set<string>();
  for (const track of p.tracks) for (const item of track.items) {
    if (ids.has(item.id)) c.addIssue({code:'custom',message:`Duplicate item ${item.id}`});
    ids.add(item.id);
    if (item.src && !p.sources[item.src]) c.addIssue({code:'custom',message:`Unknown source ${item.src}`});
    const source=item.src?p.sources[item.src]:undefined;
    if(source&&['footage','audio'].includes(item.type)){
      if(source.kind==='image'&&item.srcIn!==0)c.addIssue({code:'custom',path:['tracks',p.tracks.indexOf(track),'items',track.items.indexOf(item),'srcIn'],message:'A still image must start at source time zero'});
      if(source.kind!=='image'&&item.srcIn+item.dur>source.durationSec+1/p.settings.fps)c.addIssue({code:'custom',path:['tracks',p.tracks.indexOf(track),'items',track.items.indexOf(item),'dur'],message:`Source range exceeds ${source.durationSec} seconds`});
    }
    if (item.slotId && !p.voiceSlots.some(s=>s.id===item.slotId)) c.addIssue({code:'custom',message:`Unknown voice slot ${item.slotId}`});
  }
  for(const slot of p.voiceSlots) {
    if(slot.targetWindow.end <= slot.targetWindow.start) c.addIssue({code:'custom',message:'Voice target must have positive duration'});
    for(const take of slot.takes) if(!p.sources[take.sourceId]) c.addIssue({code:'custom',message:`Unknown take source ${take.sourceId}`});
  }
});
export type Project = z.infer<typeof ProjectSchema>;
export type JsonPatchOperation = {op:'add'|'replace'|'remove'|'test'|'move'|'copy';path:string;value?:unknown;from?:string};
export type PatchRequest = {requestId:string;baseRev:number;label:string;turnId?:string;leaseToken?:string;ops:JsonPatchOperation[]};
export type Lease = {owner:string;turnId:string;token:string;expiresAt:number;label:string};
export type HistoryEntry = {id:string;rev:number;baseRev:number;requestId:string;label:string;author:'human'|'agent'|'voice'|'system';turnId?:string;timestamp:string;ops:JsonPatchOperation[];inverse:JsonPatchOperation[];changedItemIds:string[];kind:'edit'|'undo'|'redo';target?:number};
export type Job = {id:string;kind:string;status:'queued'|'running'|'done'|'failed';progress:number;label:string;error?:string;result?:Record<string,unknown>};
export type Activity = {at:string;text:string;itemIds?:string[]};
export type Suggestion = {id:string;baseRev:number;label:string;ops:JsonPatchOperation[];status:'pending'|'accepted'|'rejected';createdAt:string;turnId?:string;commentIds?:string[];previewProject?:Project};
export type RuntimeClip = {id:string;name:string;category:string;durationSec:number;description:string;tags:string[];license:string;defaults:Record<string,unknown>;fields:Array<{key:string;label:string;type:string;min?:number;max?:number;step?:number;options?:any[]}>;[key:string]:unknown};
export type Telemetry = {contextTokens:number;baselineTokens:number;actualUsage?:{input:number;output:number;cached?:number};estimator:string;retrieval?:{total:number;byLevel:Record<string,number>;byPass:Record<string,number>}};
export type StudioState = {project:Project;lease:Omit<Lease,'token'>|null;history:HistoryEntry[];jobs:Job[];activity:Activity[];telemetry:Telemetry;transcripts:Record<string,Word[]>;provider:{fish:boolean;voiceId?:string};agentClient:'codex'|'claude';agentError?:string;clone?:{id:string;status:string};capabilities?:Record<string,boolean>;suggestions?:Suggestion[];agentPosition?:number;briefStudio?:{waiting:boolean;question?:{id:string;question:string;options:string[];answer?:string};context?:Record<string,unknown>};assetRegistry?:{clips:RuntimeClip[];moduleUrl?:string;version?:string}};
export const allItems = (p:Project) => p.tracks.flatMap(t=>t.items);
export const duration = (p:Project) => p.reel?p.reel.durationFrames/p.settings.fps:Math.max(5,...allItems(p).map(i=>i.at+i.dur));
export const findItem = (p:Project,itemId:string) => allItems(p).find(i=>i.id===itemId);
export const itemPointer = (p:Project,itemId:string) => {for(let t=0;t<p.tracks.length;t++){const i=p.tracks[t].items.findIndex(x=>x.id===itemId);if(i>=0)return `/tracks/${t}/items/${i}`;}throw new Error(`Unknown item ${itemId}`);};
export const quantize = (t:number,fps=30) => Math.round(t*fps)/fps;
export const formatTime = (t:number) => `${Math.floor(t/60).toString().padStart(2,'0')}:${(t%60).toFixed(1).padStart(4,'0')}`;
export const tokenEstimate = (data:unknown) => Math.ceil(new TextEncoder().encode(typeof data==='string'?data:JSON.stringify(data)).length/4);
export const activeTake = (slot:VoiceSlot) => slot.takes.find(t=>t.id===slot.activeTakeByMode[slot.mode]);
export const emptyProject = ():Project => ({schemaVersion:1,id:id('project'),name:'Untitled story',rev:0,settings:{fps:30,width:1080,height:1920},sources:{},tracks:[{id:'video',name:'Footage',kind:'video',items:[]},{id:'overlay',name:'Graphics',kind:'overlay',items:[]},{id:'captions',name:'Captions',kind:'caption',items:[]},{id:'voice',name:'Narration',kind:'voice',items:[]},{id:'audio',name:'Music & sound',kind:'audio',items:[]}],voiceSlots:[],comments:[]});

export function timelineWords(p:Project,transcripts:Record<string,Word[]>):Array<Word & {itemId:string;sourceId:string;sourceStart:number;sourceEnd:number}> {
  return allItems(p).filter(i=>i.type==='footage').flatMap(i=>(transcripts[i.src!]??[]).filter(w=>w.start>=i.srcIn-0.001&&w.end<=i.srcIn+i.dur+0.001).map(w=>({...w,start:i.at+w.start-i.srcIn,end:i.at+w.end-i.srcIn,itemId:i.id,sourceId:i.src!,sourceStart:w.start,sourceEnd:w.end}))).sort((a,b)=>a.start-b.start);
}
export type VoiceIntent = {kind:'local';action:'play'|'pause'|'undo'|'redo'|'seek'|'split'|'delete'|'zoom-in'|'zoom-out'|'loop';value?:number}|{kind:'confirm';action:'clear'|'delete-track'}|{kind:'comment';text:string}|{kind:'repeat';text:string};
export function routeVoice(input:string,confidence=1):VoiceIntent {
  const t=input.toLowerCase().trim().replace(/[.!?]+$/,'');
  if(confidence<.65||!t||!/[a-z0-9]/i.test(t))return {kind:'repeat',text:'I could not understand that. Please repeat or type the command.'};
  const rules:Record<string,VoiceIntent>={'play':{kind:'local',action:'play'},'pause':{kind:'local',action:'pause'},'stop':{kind:'local',action:'pause'},'undo':{kind:'local',action:'undo'},'redo':{kind:'local',action:'redo'},'split here':{kind:'local',action:'split'},'delete selected':{kind:'local',action:'delete'},'zoom in':{kind:'local',action:'zoom-in'},'zoom out':{kind:'local',action:'zoom-out'},'loop this section':{kind:'local',action:'loop'},'clear all':{kind:'confirm',action:'clear'},'delete all':{kind:'confirm',action:'clear'}};
  if(rules[t])return rules[t];
  if(/\b(?:delete|remove|clear|erase)\b.*\btrack\b/.test(t))return {kind:'confirm',action:'delete-track'};
  if(/\b(?:delete|remove|clear|erase)\b.*\b(?:all|entire|everything|whole)\b/.test(t))return {kind:'confirm',action:'clear'};
  const m=t.match(/^(?:go to|seek to|jump to)\s+(\d+(?:\.\d+)?)\s*(?:seconds?|s)?$/);if(m)return {kind:'local',action:'seek',value:Number(m[1])};
  if(!/^(?:please\s+)?(?:make|change|set|increase|decrease|add|remove|move|trim|speed|slow|clean|create|generate|replace|restore|color|resize|highlight|caption|mute|unmute|adjust|animate|apply|convert|rewrite|edit|turn|fix|cut|shorten|extend|put|give|show|use|rearrange)\b/.test(t)||t.split(/\s+/).length<2)return {kind:'repeat',text:'That does not look like an editing request. Please repeat or type a specific edit.'};
  return {kind:'comment',text:input.trim()};
}
