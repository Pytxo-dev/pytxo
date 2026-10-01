import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

for (const theme of ["void", "light"]) test(`required action readable in ${theme}`, async ({ page }) => {
  await completeOnboarding(page, { "pytxo-deck-theme": theme });
  await page.setViewportSize({ width:860,height:760 }); await page.goto('/#/work');
  const label=page.locator('.attention-action strong'); await expect(label).toHaveText('Decision needed');
  const contrast=await label.evaluate(node=>{
    const luminance=(color:string)=>{const channels=color.match(/[\d.]+/g)!.slice(0,3).map(Number).map(v=>{v/=255;return v<=.04045?v/12.92:((v+.055)/1.055)**2.4});return channels[0]*.2126+channels[1]*.7152+channels[2]*.0722};
    const fg=luminance(getComputedStyle(node).color),bg=luminance(getComputedStyle(node.closest('button')!).backgroundColor);return (Math.max(fg,bg)+.05)/(Math.min(fg,bg)+.05);
  }); expect(contrast).toBeGreaterThanOrEqual(4.5);
});

test('laptop Review keeps expanded map and substantial comparison above decision',async({page})=>{
 await completeOnboarding(page,{'pytxo-preview-candidate-check-v1':'passed','pytxo-preview-review-substantial-v1':'true'});
 await page.setViewportSize({width:1280,height:720});await page.goto('/#/history');await page.locator('.row[data-run-id="run-71ad"]').click();await page.locator('.outcome-action').click();
 await expect(page.locator('.candidate-overview')).toHaveAttribute('open','');await expect(page.locator('.text-content').first()).toContainText('preserves_public_shape');
 const comparison=(await page.locator('.exact-diff').boundingBox())!,decision=(await page.locator('.decision-bar').boundingBox())!;
 expect(Math.min(comparison.y+comparison.height,decision.y)-Math.max(comparison.y,0)).toBeGreaterThanOrEqual(240);
 await expect(page.getByRole('button',{name:'Apply reviewed changes',exact:true})).toBeEnabled();
 await page.getByText('Focus on code',{exact:true}).click();await expect(page.locator('.candidate-overview')).not.toHaveAttribute('open','');
});

for(const width of [1280,860]) test(`selected inspector remains connected at ${width}`,async({page})=>{
 await completeOnboarding(page);await page.setViewportSize({width,height:760});await page.goto('/#/work');
 const map=page.getByTestId('execution-map');
 await map.locator('.task-node[data-state="queued"] button').click();await expect(map.locator('.task-node.chosen')).toHaveAttribute('data-state','queued');
 const inspector=page.locator('.dock-panel:visible:not(.output-panel)');await expect(inspector).toBeInViewport();await expect(inspector).toContainText('Recorded scope');
 const task=(await map.boundingBox())!,detail=(await inspector.boundingBox())!;
 if(width===860 || page.viewportSize()!.height<780){
  expect(detail.y).toBeGreaterThanOrEqual(task.y+task.height-1);
  await expect(page.getByRole('tablist',{name:'bottom dock'})).toBeVisible();
 }else{expect(detail.x).toBeGreaterThan(task.x);expect(detail.y).toBeLessThan(task.y+40);}
 await expect(map.locator('.commit-rail')).toHaveCount(0);
 await expect(page.getByRole('button',{name:'Review changes',exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Stop',exact:true})).toBeEnabled();
});

test('History separates unlinked candidate evidence from integration record',async({page})=>{
 await completeOnboarding(page,{'pytxo-preview-review-state-v1':'applied','pytxo-preview-candidate-check-v1':'passed'});await page.goto('/#/history');await page.locator('.row[data-run-id="run-71ad"]').click();
 await expect(page.locator('.identity-gap')).toBeVisible();await expect(page.locator('.integration-record')).toContainText('Apply record found');
 await expect(page.locator('.integration-record')).not.toContainText('Recorded checks passed');await expect(page.locator('.outcome-action')).toHaveText('View applied result');
 await page.locator('.trace-lane.integration').getByRole('button',{name:/4 prepared files/}).click();
 await expect(page.locator('.candidate-inventory')).toHaveAttribute('open','');await expect(page.locator('.candidate-inventory')).toContainText('assets/signal-mark.bin');
 await expect(page.locator('.files-section > header')).toContainText('3');
});
