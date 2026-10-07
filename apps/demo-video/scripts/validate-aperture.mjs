import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import ts from 'typescript';

const app=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const repo=path.resolve(app,'../..');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const inside=(root,relative)=>{
  const resolved=path.resolve(root,relative);
  if(!resolved.startsWith(root+path.sep))throw Error(`Artifact escapes its root: ${relative}`);
  return resolved;
};
const read=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const source=await fs.readFile(path.join(app,'src/aperture/config.ts'),'utf8');
const js=ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText;
const {parseApertureFilmProps}=await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
const props=parseApertureFilmProps(await read(path.join(app,'aperture-props.json')));
const e=props.evidence;
const evidence=await read(inside(repo,e.evidenceLocation));
for(const role of ['repository','independent']){
  const command=evidence.post_apply_verification?.[role];
  if(command?.exit_code!==0||!/^([a-f0-9]{64})$/.test(command.log_sha256??''))throw Error(`${role} needs its passing process outcome and log hash`);
}
if(!/^[a-f0-9]{64}$/.test(evidence.post_apply_verification.independent.verifier_sha256??'')||e.capturedAt!==evidence.capture_date)throw Error('Verifier hash or capture date missing');
const planBytes=await fs.readFile(inside(repo,e.dependencyEvidence.artifact));
const plan=JSON.parse(planBytes);
if(hash(planBytes)!==e.dependencyEvidence.sha256||evidence.plan_sha256!==e.dependencyEvidence.sha256||JSON.stringify(plan)!==JSON.stringify(evidence.plan))throw Error('Planner artifact differs from the recorded run');
for(const [role,mapping] of Object.entries(e.dependencyEvidence.tasks)){
  const task=plan.waves.flat().find(t=>t.task_id===mapping.id);
  if(!task||JSON.stringify(task.depends_on)!==JSON.stringify(mapping.dependsOn)||JSON.stringify(task.paths)!==JSON.stringify([e.scopePaths[role]]))throw Error(`Diagram ${role} differs from the recorded task`);
}
if(evidence.run_id!==e.runId||evidence.packaged_executable_sha256!==e.executableSha256||evidence.package_digest!==e.applyPackageId||evidence.apply.state!=='committed'||evidence.apply.independent_tests_passed!==e.independentChecksPassed||evidence.apply.repository_tests_passed!==e.repositoryTestsPassed||evidence.apply.changed_hashes_match!==e.changedFilesMatchingPackage||evidence.apply.receipt_survived_restart!==e.receiptSurvivedRestart||!evidence.primary_before_apply.inventory_matches||!evidence.primary_before_apply.cancel_left_unchanged||evidence.candidate_checks.length!==e.combinedChecksPassed||evidence.candidate_checks.some(c=>!c.passed))throw Error('Film claims differ from the Bench record');
for(const [role,clip] of Object.entries(props.clips)){
  const file=inside(path.join(app,'public'),clip.src);
  if(hash(await fs.readFile(file))!==clip.sha256)throw Error(`${role} footage hash mismatch`);
  const result=spawnSync('ffprobe',['-v','error','-show_streams','-show_format','-of','json',file],{encoding:'utf8'});
  if(result.error||result.status!==0)throw result.error??Error(`Cannot inspect ${role} footage`);
  const metadata=JSON.parse(result.stdout);const video=metadata.streams.filter(s=>s.codec_type==='video');
  if(video.length!==1||metadata.streams.some(s=>s.codec_type==='audio'))throw Error(`${role} must contain one silent video stream`);
  const [num,den]=video[0].avg_frame_rate.split('/').map(Number);
  if(video[0].width!==clip.width||video[0].height!==clip.height||Math.abs(num/den-clip.sourceFps)>0.001||Math.abs(Number(metadata.format.duration)-clip.sourceDurationSeconds)>0.02)throw Error(`${role} footage metadata mismatch`);
  if(JSON.stringify(clip)!==JSON.stringify(evidence.clips[role]))throw Error(`${role} edit differs from the Bench record`);
}
console.log(`Aperture assets verified: ${e.runId}; exact planner graph, 3 video hashes, metadata and recorded claims match. This checks consistency, not independent execution.`);
