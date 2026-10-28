/** App turns receive only the tool family needed for that request. External clients default to all. */
const base=['export_start','project_patch','project_get','item_update','item_split','item_move','item_duplicate','item_delete','comments_list','comments_bundle','comments_resolve','comments_reply','job_status','history_list','scene_read','scene_draft','scene_status','reel_read'];
export const toolProfiles:Record<string,string[]>={
 creative:[...base.filter(t=>!['export_start','comments_resolve','comments_reply'].includes(t)),'storyboard_set','storyboard_read','library_list','library_inspect','library_place','motion_read','reel_plan_set','reel_chapter_score_set'],
 outline:['video_outline_set','reel_read','project_get','library_list','footage_pack','motion_read'],
 direction:['video_story_set','video_choreography_set','video_plan_read','motion_reference','motion_search','motion_read','footage_pack','footage_view','library_list'],
 story:['video_story_set','video_plan_read','motion_search','motion_read','footage_pack','library_list'],
 choreography:['video_plan_read','video_choreography_set','motion_search','motion_read'],
 production:[...base,'library_list','library_inspect','library_place','footage_pack','footage_view','motion_reference','video_plan_read','motion_search','motion_read','reel_plan_set','reel_chapter_score_set'],
 diagnose:['video_edit_decision_set','video_review_frames','video_plan_read','project_get','scene_read','footage_pack','footage_view','motion_search'],
 edit:[...base,'video_plan_read','motion_search','motion_read'],
 review:['video_review_frames','video_review_submit','video_plan_read'],
 'scene-edit':base,
 authoring:[...base,'motion_search','motion_read','reel_plan_set','reel_chapter_score_set'],
 media:[...base,'footage_pack','footage_view','footage_candidates','footage_delete_words','footage_transcribe','library_list','library_import','library_place'],
 voice:[...base,'voice_script','voice_generate','voice_mode','voice_fit','voice_clone','voice_window','voice_convert_all'],
};
export function chooseToolProfile(prompt:string,context:any){if(/\b(narration|clon(?:e|ing)|voice|speech)\b/i.test(prompt))return 'voice';if(/\b(transcript|interview|filler|silence|footage|talking.head)\b/i.test(prompt))return 'media';if(context?.layers?.some((i:any)=>i.sceneVersion)&&!/^\s*(create|generate|build)\b/i.test(prompt))return 'scene-edit';return 'authoring'}

const retrievalTools=['knowledge_search','knowledge_card','knowledge_skeleton','knowledge_slice','knowledge_source','taste_search','taste_read','pipeline_detect','pipeline_status','quality_verify','quality_frames','retrieval_meter'];
for(const [profile,tools] of Object.entries(toolProfiles)){if(profile!=='review')tools.push(...retrievalTools);if(['creative','production','authoring','media','voice','edit'].includes(profile))tools.push('knowledge_derive','knowledge_instantiate','pipeline_plan','pipeline_base','pipeline_pass','pipeline_resume','pipeline_handoff');}
