import { expect, test } from '@playwright/test';
import { completeOnboarding } from './helpers';
test('History separates outcomes and supports narrow list to detail with focus restoration', async ({page}) => {
 await completeOnboarding(page); await page.setViewportSize({width:1600,height:1000}); await page.goto('/#/history');
 await expect(page.locator('.history .row')).toHaveCount(2);
 await page.locator('.history .row[data-run-id="run-71ad"]').click();
 await expect(page.locator('.outcome-heading h2')).toHaveText('Work in signal-lab'); await expect(page.locator('.files-section')).toContainText('crates/pytxo-signal/src/lib.rs');
 await expect(page.locator('.outcome-heading')).toContainText('No confirmed Apply'); await expect(page.locator('.recorded-trace')).toContainText('No Apply record'); await expect(page.locator('.recorded-trace')).toContainText('not a complete timeline');
 await expect(page.getByText('No saved request text. The repository name is used as a fallback.')).toHaveCount(0);
 for (const identity of await page.getByText('pkg-71ad-immutable').all()) await expect(identity).toBeHidden(); await page.getByText('Technical details',{exact:true}).click(); await expect(page.getByText('pkg-71ad-immutable').first()).toBeVisible();
 await page.getByLabel('Search work history').fill('absent-request'); await expect(page.getByText('No runs match')).toBeVisible(); await page.getByLabel('Search work history').fill('');
 await page.setViewportSize({width:860,height:700}); await page.getByRole('button',{name:'Back to runs'}).click(); await expect(page.locator('.run-navigator')).toBeVisible();
 await page.locator('.history .row').first().click(); await expect(page.locator('.outcome-inspector')).toBeVisible(); await page.getByRole('button',{name:'Back to runs'}).click(); await expect(page.locator('.history .row').first()).toBeFocused();
});
test('History ignores a late response after another selection',async({page})=>{
 await completeOnboarding(page,{'pytxo-preview-native-agent-ids-v1':'switch','pytxo-preview-state-matrix-v1':'1'}); await page.goto('/#/history'); await expect(page.locator('.history > .unresolved')).toBeVisible();
 await page.locator('.history .row[data-run-id="run-other"]').click(); await expect(page.locator('.history > .unresolved')).toHaveCount(0,{timeout:1000});
 await page.locator('.history .row[data-run-id="run-71ad"]').click(); await page.getByText('Technical details',{exact:true}).click(); await expect(page.locator('.technical')).toContainText('pkg-71ad'); await page.waitForTimeout(1800); await expect(page.locator('.technical')).toContainText('pkg-71ad');
});
for(const theme of ['void','light']) test(`History larger list in ${theme}`,async({page})=>{
 await completeOnboarding(page,{'pytxo-preview-layout-fixture-v1':'long','pytxo-preview-flow-history-v1':'long-mission','pytxo-deck-theme':theme}); await page.emulateMedia({reducedMotion:'reduce'}); await page.goto('/#/history'); await expect(page.locator('.history .row')).toHaveCount(42); await expect(page.locator('.run-navigator')).toBeVisible();
});
test('Work selection differs from active brackets',async({page})=>{
 await completeOnboarding(page); await page.setViewportSize({width:1600,height:1000}); await page.goto('/#/work'); await expect(page.locator('.work h1')).toHaveText('Work in pytxo');
 const map=page.getByTestId('execution-map'); await map.getByRole('button',{name:/^tests /}).click(); await expect(map.locator('.task-node.chosen')).not.toHaveClass(/running/); await expect(map.locator('.task-node.running')).toContainText('ui');
 await map.locator('.task-node').getByRole('button',{name:/^ui /}).focus(); await page.keyboard.press('Enter'); await expect(map.locator('.task-node.chosen')).toHaveClass(/running/); await expect(page.getByRole('button',{name:'Stop',exact:true})).toBeEnabled();
});
test('Work promotes only approvals bound to the focused run',async({page})=>{
 await completeOnboarding(page,{'pytxo-preview-approval-scope-v1':'foreign-run'}); await page.goto('/#/work');
 await expect(page.getByRole('button',{name:/Decision needed/})).toHaveCount(0);
 await expect(page.locator('.work-heading').getByText('Running',{exact:true})).toBeVisible();
 await expect(page.locator('.aperture-glyph')).toHaveAttribute('data-active','true');
 await expect(page.getByRole('button',{name:'Review changes',exact:false})).toBeEnabled();
});
test('empty history shows no invented records',async({page})=>{await completeOnboarding(page,{'pytxo-preview-history-empty-v1':'1'});await page.goto('/#/history');await expect(page.getByText('No recorded runs',{exact:true})).toBeVisible();await expect(page.locator('.recorded-trace')).toHaveCount(0);});
for(const state of ['ready','applied','recovery_required','unavailable']) test(`History ${state} preserves outcome distinctions`,async({page})=>{await completeOnboarding(page,{'pytxo-preview-review-state-v1':state,'pytxo-preview-candidate-check-v1':'passed','pytxo-preview-history-evidence-error-v1':state==='unavailable'?'1':'0'});await page.goto('/#/history');await page.locator('.history .row[data-run-id="run-71ad"]').click();if(state==='unavailable'){await expect(page.getByText(/Evidence unavailable:/)).toBeVisible(); await expect(page.locator('.files-section')).toContainText('No file inventory available');await expect(page.locator('.files-section')).not.toContainText('pkg-8f2c');}else{await expect(page.locator('.outcome-action')).toHaveText(state==='applied'?'View applied result':state==='recovery_required'?'Inspect recovery':'Review prepared changes');await expect(page.locator('.recorded-trace .confirmed')).toHaveCount(state==='applied'?1:0);}});
test('saved task prose is visible while task identity and selected context remain accessible',async({page})=>{await completeOnboarding(page,{'pytxo-preview-flow-history-v1':'long-mission'});await page.goto('/#/work');const map=page.getByTestId('execution-map');await expect(map.locator('.task-node.chosen')).toContainText('Clarify the candidate review experience');await map.getByRole('button',{name:/^ui /}).click();const detail=page.locator('.dock-panel:visible:not(.output-panel)');await detail.locator('.source > summary').click();await expect(detail.locator('.source')).toContainText('Agent desktop');await expect(detail.locator('header strong').first()).toHaveText('Clarify the candidate review experience');});
test('stopped execution retains its prepared candidate without claiming integration',async({page})=>{await completeOnboarding(page);await page.goto('/#/work');await page.getByRole('button',{name:'Stop',exact:true}).click();await page.getByRole('button',{name:'Stop run',exact:true}).click();await page.getByRole('link',{name:'History',exact:true}).click();await expect(page.locator('.outcome-heading')).toContainText('Stopped');await expect(page.locator('.outcome-action')).toHaveText('Review prepared changes');await expect(page.locator('.recorded-trace .confirmed')).toHaveCount(0);});
test('confirmed rollback is recorded without claiming an unresolved recovery or successful Apply',async({page})=>{await completeOnboarding(page,{'pytxo-preview-review-state-v1':'recovered'});await page.goto('/#/history');await page.locator('.history .row[data-run-id="run-71ad"]').click();await expect(page.locator('.outcome-heading')).toContainText('Rollback confirmed');await expect(page.locator('.outcome-action')).toHaveText('Review prepared changes');await expect(page.locator('.recorded-trace .confirmed')).toHaveCount(0);});
