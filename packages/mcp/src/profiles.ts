/** App turns receive only the tool family needed for that request. External clients default to all. */
const base=['export_start','project_patch','project_get','item_update','item_split','item_move','item_duplicate','item_delete','comments_list','comments_bundle','comments_resolve','comments_reply','job_status','history_list','scene_read','scene_draft','scene_status','reel_read'];
export const toolProfiles:Record<string,string[]>={
 'scene-edit':base,
 authoring:[...base,'motion_search','motion_read','reel_plan_set','reel_chapter_score_set'],
 media:[...base,'footage_pack','footage_view','footage_candidates','footage_delete_words','footage_transcribe','library_list','library_import','library_place'],
 voice:[...base,'voice_script','voice_generate','voice_mode','voice_fit','voice_clone','voice_window','voice_convert_all'],
};
export function chooseToolProfile(prompt:string,context:any){if(/\b(narration|clon(?:e|ing)|voice|speech)\b/i.test(prompt))return 'voice';if(/\b(transcript|interview|filler|silence|footage|talking.head)\b/i.test(prompt))return 'media';if(context?.layers?.some((i:any)=>i.sceneVersion)&&!/^\s*(create|generate|build)\b/i.test(prompt))return 'scene-edit';return 'authoring'}
