import {test,expect} from '@playwright/test';

// Real backend only. These checks never invoke an external agent or paid voice provider.
test('draft preview, one-commit sliders, range comments and selective history share the live project',async({page,request})=>{
 const {token}=await (await request.get('/api/bootstrap')).json();
 const headers={'X-ClipKit-Token':token};
 const op=async(name:string,args:Record<string,unknown>={})=>{const response=await request.post(`/api/op/${name}`,{headers,data:args});expect(response.ok(),await response.text()).toBeTruthy();return response.json()};
 const state=async()=>await (await request.get('/api/state',{headers})).json();
 await op('demo_reset');const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));await page.goto('/');
 await page.getByLabel('Voice or typed editing command').fill('go to 2');await page.getByLabel('Voice or typed editing command').press('Enter');
 await page.getByRole('button',{name:/Kinetic title, /}).click();await expect(page.locator('.selection-outline')).toBeVisible();
 const revision=(await state()).project.rev;const text=page.getByRole('textbox',{name:'Text',exact:true});await text.fill('Live draft preview');
 await expect(page.locator('.stage [data-ck-path="text"]').filter({hasText:'Live draft preview'})).toBeVisible();expect((await state()).project.rev).toBe(revision);
 await text.blur();await expect.poll(async()=>(await state()).project.rev).toBe(revision+1);
 const slider=page.getByLabel('Type size slider',{exact:true});await slider.focus();const sliderRev=(await state()).project.rev;await slider.press('ArrowRight');await expect.poll(async()=>(await state()).project.rev).toBe(sliderRev+1);await slider.blur();expect((await state()).project.rev).toBe(sliderRev+1);
 await page.getByRole('button',{name:'Select tool',exact:true}).focus();const count=(await state()).project.tracks.flatMap((track:any)=>track.items).length;await page.keyboard.press('d');await expect.poll(async()=>(await state()).project.tracks.flatMap((track:any)=>track.items).length).toBe(count+1);await page.keyboard.press('Control+Shift+Z'); // No redo branch should mutate when nothing was undone.
 await page.keyboard.press('Control+z');await expect.poll(async()=>(await state()).project.tracks.flatMap((track:any)=>track.items).length).toBe(count);
 const ruler=await page.locator('.ruler-marks').boundingBox();expect(ruler).toBeTruthy();await page.keyboard.down('Shift');await page.mouse.move(ruler!.x+55,ruler!.y+12);await page.mouse.down();await page.mouse.move(ruler!.x+165,ruler!.y+12,{steps:6});await page.mouse.up();await page.keyboard.up('Shift');await expect(page.locator('.timeline-range')).toBeVisible();
 await page.getByRole('button',{name:'Comments',exact:true}).click();await page.getByLabel('New comment').fill('Review this selected range');await page.getByRole('button',{name:'Add comment',exact:true}).click();await expect(page.getByRole('button',{name:'Comment: Review this selected range'})).toBeVisible();const saved=(await state()).project.comments.find((comment:any)=>comment.text==='Review this selected range');expect(saved.anchor.tEnd).toBeGreaterThan(saved.anchor.tStart);
 await page.getByRole('button',{name:'History',exact:true}).click();await page.locator('.history-select').first().click();await expect(page.locator('.history-entry').first()).toContainText('human');
 const lastRev=(await state()).project.rev;await page.locator('.history-entry').first().getByRole('button',{name:'Revert this edit'}).click();await expect.poll(async()=>(await state()).project.rev).toBe(lastRev+1);expect((await state()).project.comments.some((comment:any)=>comment.id===saved.id)).toBeFalsy();
 expect(errors).toEqual([]);
});

test('voice fallback and export settings stay truthful at desktop widths',async({page,request})=>{
 const {token}=await (await request.get('/api/bootstrap')).json();const headers={'X-ClipKit-Token':token};
 await request.post('/api/op/demo_reset',{headers,data:{}});await page.goto('/');
 await page.getByRole('button',{name:'Voice',exact:true}).click();await page.getByLabel('Voice script').fill('First narration line.\nSecond narration line.');await page.getByRole('button',{name:'Create voice slots'}).click();await expect(page.locator('.voice-card')).toHaveCount(2);
 await expect(page.getByRole('button',{name:'Record whole script'})).toBeVisible();await expect(page.getByRole('button',{name:'Convert all to my clone'})).toBeDisabled();
 const current=await (await request.get('/api/state',{headers})).json();if(!current.provider.fish){await expect(page.getByRole('button',{name:'Browser voice · preview only'})).toHaveCount(2);await expect(page.getByText('Temporary browser speech is never included in exports.').first()).toBeVisible()}
 await page.getByRole('button',{name:'Export video',exact:true}).click();const dialog=page.getByRole('dialog',{name:'Export video settings'});await expect(dialog).toBeVisible();await page.getByLabel('Export preset').selectOption('landscape');await page.getByLabel('Export quality').selectOption('high');await expect(dialog).toContainText('Used assets & licenses');const exportRequest=page.waitForRequest(request=>request.url().endsWith('/api/op/export_start')&&request.method()==='POST');await page.getByRole('button',{name:'Render video',exact:true}).click();expect((await exportRequest).postDataJSON()).toEqual({preset:'landscape',quality:'high'});await expect(page.locator('.toast')).toContainText(/record|generate|narration/i);
 for(const width of [1280,1920]){await page.setViewportSize({width,height:1000});await expect(page.getByRole('button',{name:'Export video',exact:true})).toBeInViewport();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();await page.screenshot({path:`.clipkit/verification/studio-audit-${width}.png`,fullPage:true})}
 await page.getByLabel('Voice or typed editing command').fill('flobble zzz');await page.getByLabel('Voice or typed editing command').press('Enter');await expect(page.locator('.heard-chip')).toContainText(/repeat/i);const after=await (await request.get('/api/state',{headers})).json();expect(after.project.comments).toHaveLength(0);
});

test('native image and easing inspector controls use an actual imported image source',async({page,request})=>{
 const {readFile}=await import('node:fs/promises');
 const {token}=await (await request.get('/api/bootstrap')).json();const headers={'X-ClipKit-Token':token};
 await request.post('/api/op/demo_reset',{headers,data:{}});await request.post('/api/op/assets_add',{headers,data:{clipId:'screenshot-frame',at:2}});
 await page.goto('/');await page.getByLabel('Voice or typed editing command').fill('go to 3');await page.getByLabel('Voice or typed editing command').press('Enter');await page.getByRole('button',{name:/Screenshot frame, /}).click();
 await page.getByLabel('Easing',{exact:true}).selectOption('ease-in-out');
 await page.getByLabel('Upload Screenshot image').setInputFiles({name:'native-preview-still.png',mimeType:'image/png',buffer:await readFile('public/previews/counter.png')});
 await expect.poll(async()=>{const current=await (await request.get('/api/state',{headers})).json();const frame=current.project.tracks.flatMap((track:any)=>track.items).find((item:any)=>item.clipId==='screenshot-frame');return current.project.sources[frame.props.imageSourceId]?.kind}).toBe('image');
 const current=await (await request.get('/api/state',{headers})).json();const frame=current.project.tracks.flatMap((track:any)=>track.items).find((item:any)=>item.clipId==='screenshot-frame');expect(frame.props.easing).toBe('ease-in-out');await expect(page.getByLabel('Screenshot image',{exact:true})).toHaveValue(frame.props.imageSourceId);
});

test('asset usage/provenance inspection is read-only, keyboard accessible and available while locked',async({page,request})=>{
 const {token}=await (await request.get('/api/bootstrap')).json();const headers={'X-ClipKit-Token':token};await request.post('/api/op/demo_reset',{headers,data:{graphics:true}});await page.goto('/');const before=await (await request.get('/api/state',{headers})).json();
 const about=page.getByRole('button',{name:'About Kinetic title',exact:true});await about.click();const dialog=page.getByRole('dialog',{name:'Asset usage and provenance'});await expect(dialog).toBeVisible();await expect(dialog).toContainText('agent-authored');await expect(dialog).toContainText('Best for');await expect(dialog).toContainText('Frame checks passed locally');expect((await (await request.get('/api/state',{headers})).json()).project.rev).toBe(before.project.rev);await page.keyboard.press('Escape');await expect(dialog).toBeHidden();await expect(about).toBeFocused();
 const lease=await (await request.post('/api/op/status_working',{headers,data:{owner:'asset-inspection-test',label:'Read only'}})).json();try{await expect(page.getByRole('button',{name:'Add Kinetic title',exact:true})).toBeDisabled();await about.click();await expect(dialog).toBeVisible();await expect(dialog.getByRole('button',{name:'Add at playhead'})).toBeDisabled();await page.screenshot({path:'.clipkit/verification/asset-details.png'});await page.keyboard.press('Escape');}finally{await request.post('/api/op/status_idle',{headers,data:{leaseToken:lease.token}})}
});
