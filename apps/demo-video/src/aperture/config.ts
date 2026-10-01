/** All time values in the capture manifest are source-video seconds. */
export type NativeClip = {
  /** Relative to apps/demo-video/public. No remote assets or screenshot fallback. */
  src: string;
  sha256: string;
  width: number;
  height: number;
  sourceFps: number;
  sourceDurationSeconds: number;
  trimStartSeconds: number;
  playbackRate: number;
  /** Time cuts in this encoded source video, measured from the file start. */
  cuts?: {atSeconds: number; omittedSeconds?: number}[];
  /** Optional editorial crop, in normalized source coordinates. No synthetic UI. */
  focus?: {x: number; y: number; zoom: number};
};

export type ApertureEvidence = {
  runId: string;
  buildLabel: string;
  executableSha256: string;
  capturedAt: string;
  harness: string;
  harnessVersion: string;
  /** An actual published evidence URL, or a visible local record path. */
  evidenceLocation: string;
  evidenceLocationLabel: string;
  plannedTasks: 3;
  waves: 2;
  maxConcurrentWorkers: 2;
  combinedChecksPassed: number;
  independentChecksPassed: number;
  repositoryTestsPassed: number;
  changedFilesMatchingPackage: number;
  primaryUnchangedBeforeApply: true;
  applyCommitted: true;
  receiptSurvivedRestart: boolean;
  applyPackageId: string;
  scopePaths: {code: string; docs: string; tests: string};
  /** Actual planner record: wave order alone does not establish task dependencies. */
  dependencyEvidence: {
    artifact: string;
    sha256: string;
    tasks: Record<'code' | 'docs' | 'tests', {id: string; dependsOn: string[]}>;
  };
  /** Describe observed limitations in the linked record. Not a product guarantee. */
  recordingMethod: string;
};

export type ApertureFilmProps = {
  evidence: ApertureEvidence;
  clips: {review: NativeClip; apply: NativeClip; result: NativeClip};
  /** Editorial masks, separate from the unmodified native evidence record. */
  privacyEdits: Record<'review' | 'apply' | 'result', NativePrivacyEdit>;
};

export type NativePrivacyEdit = {
  sourceSha256: string;
  regions: {
    /** Inclusive/exclusive indices in the encoded source video. */
    fromFrame: number;
    untilFrame: number;
    x: number;
    y: number;
    width: number;
    height: number;
    label: 'Host path redacted';
  }[];
};

export const FILM_FPS = 60;
export const FILM_DURATION = 58 * FILM_FPS;
export const CLIP_SECONDS = {review: 13, apply: 11, result: 8} as const;

const fail = (message: string): never => {
  throw new Error(`PytxoApertureFilm capture manifest: ${message}`);
};

const record = (value: unknown, label: string): Record<string, unknown> => {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) fail(`${label} must be an object`);
  return value as Record<string, unknown>;
};
const observedString = (value: unknown, label: string): string => {
  if (typeof value !== 'string' || !value.trim() || /REPLACE|TODO|PENDING/i.test(value)) fail(`${label} must come from the observed run`);
  return value as string;
};
const digest = (value: unknown, label: string): void => {
  if (typeof value !== 'string' || !/^[a-f0-9]{64}$/i.test(value)) fail(`${label} must be the recorded SHA-256`);
};
const number = (value: unknown, label: string): number => {
  if (typeof value !== 'number' || !Number.isFinite(value)) fail(`${label} must be a finite number`);
  return value as number;
};

/** Accept imported JSON as unknown; narrow only after every required field is checked. */
export function assertApertureEvidence(value: unknown): asserts value is ApertureFilmProps {
  const props = record(value, 'manifest');
  const e = record(props.evidence, 'evidence');
  const clips = record(props.clips, 'clips');
  const privacyEdits = record(props.privacyEdits, 'privacyEdits');
  for (const key of ['runId', 'buildLabel', 'capturedAt', 'harness', 'harnessVersion', 'evidenceLocation', 'evidenceLocationLabel', 'applyPackageId', 'recordingMethod'] as const) {
    observedString(e[key], key);
  }
  digest(e.executableSha256, 'executableSha256');
  if (e.plannedTasks !== 3 || e.waves !== 2 || e.maxConcurrentWorkers !== 2) fail('this film depicts three tasks in two waves with two-worker capacity; revise the structure scene for another run');
  if (e.primaryUnchangedBeforeApply !== true || e.applyCommitted !== true) fail('the depicted pre-Apply and committed outcome must be independently observed');
  if (typeof e.receiptSurvivedRestart !== 'boolean') fail('receiptSurvivedRestart must be an observed boolean');
  for (const key of ['combinedChecksPassed', 'independentChecksPassed', 'repositoryTestsPassed', 'changedFilesMatchingPackage'] as const) {
    const count = number(e[key], key);
    if (!Number.isInteger(count) || count < 1) fail(`${key} requires a positive observed count`);
  }
  const scopePaths = record(e.scopePaths, 'scopePaths');
  for (const key of ['code', 'docs', 'tests'] as const) {
    if (observedString(scopePaths[key], `scopePaths.${key}`).length > 46) fail(`scopePaths.${key} needs a readable actual scope (maximum 46 characters)`);
  }

  const dependencyEvidence = record(e.dependencyEvidence, 'dependencyEvidence');
  observedString(dependencyEvidence.artifact, 'dependencyEvidence.artifact');
  digest(dependencyEvidence.sha256, 'dependencyEvidence.sha256');
  const tasks = record(dependencyEvidence.tasks, 'dependencyEvidence.tasks');
  const mapped = Object.fromEntries((['code', 'docs', 'tests'] as const).map((role) => {
    const task = record(tasks[role], `dependencyEvidence.tasks.${role}`);
    const id = observedString(task.id, `dependencyEvidence.tasks.${role}.id`);
    if (!Array.isArray(task.dependsOn)) fail(`dependencyEvidence.tasks.${role}.dependsOn must be the actual task-ID array`);
    const dependsOn = (task.dependsOn as unknown[]).map((dependency) => observedString(dependency, `dependencyEvidence.tasks.${role}.dependsOn`));
    if (new Set(dependsOn).size !== dependsOn.length) fail(`dependencyEvidence.tasks.${role} contains duplicate dependencies`);
    return [role, {id, dependsOn}];
  })) as ApertureEvidence['dependencyEvidence']['tasks'];
  if (new Set(Object.values(mapped).map((task) => task.id)).size !== 3) fail('the three diagram roles require distinct actual task IDs');
  if (mapped.code.dependsOn.length !== 0 || mapped.docs.dependsOn.length !== 0 || mapped.tests.dependsOn.length !== 1 || mapped.tests.dependsOn[0] !== mapped.code.id) {
    fail('this diagram requires independent Code and Docs tasks, with Tests depending only on Code; revise the diagram for another observed dependency graph');
  }

  for (const key of ['review', 'apply', 'result'] as const) {
    const clip = record(clips[key], `clips.${key}`);
    const src = observedString(clip.src, `${key}.src`);
    if (/^(?:[a-z]+:|[\\/])|\.\./i.test(src)) fail(`${key}.src must name the actual local public/ video`);
    if (!/\.(?:mp4|webm|mov)$/i.test(src)) fail(`${key} requires a video`);
    digest(clip.sha256, `${key}.sha256`);
    for (const field of ['width', 'height', 'sourceFps'] as const) {
      if (number(clip[field], `${key}.${field}`) <= 0) fail(`${key}.${field} must be positive`);
    }
    const sourceDuration = number(clip.sourceDurationSeconds, `${key}.sourceDurationSeconds`);
    const rate = number(clip.playbackRate, `${key}.playbackRate`);
    const start = number(clip.trimStartSeconds, `${key}.trimStartSeconds`);
    if (sourceDuration <= 0 || start < 0) fail(`${key} duration must be positive and trim start nonnegative`);
    if (rate < 0.5 || rate > 4) fail(`${key} playback is limited to 0.5–4×; prefer 1× for Review and Apply`);
    const end = start + CLIP_SECONDS[key] * rate;
    if (end > sourceDuration + 0.001) fail(`${key} needs source footage through ${end.toFixed(3)} seconds; do not loop or freeze a short capture`);
    if (clip.focus !== undefined) {
      const focus = record(clip.focus, `${key}.focus`);
      const x = number(focus.x, `${key}.focus.x`);
      const y = number(focus.y, `${key}.focus.y`);
      const zoom = number(focus.zoom, `${key}.focus.zoom`);
      if (x < 0 || x > 1 || y < 0 || y > 1 || zoom < 1 || zoom > 2) fail(`${key}.focus must be in bounds, with 1–2× editorial zoom`);
    }
    if (clip.cuts !== undefined && !Array.isArray(clip.cuts)) fail(`${key}.cuts must be an array`);
    for (const entry of (clip.cuts ?? []) as unknown[]) {
      const cut = record(entry, `${key}.cuts entry`);
      const at = number(cut.atSeconds, `${key}.cuts.atSeconds`);
      if (at < 0 || at > sourceDuration) fail(`${key} cut must have an actual in-bounds source timestamp`);
      if (cut.omittedSeconds !== undefined && number(cut.omittedSeconds, `${key}.cuts.omittedSeconds`) <= 0) fail(`${key} omitted duration must be measured and positive, or omitted when unknown`);
    }
    const privacy = record(privacyEdits[key], `privacyEdits.${key}`);
    if (privacy.sourceSha256 !== clip.sha256) fail(`${key} privacy edits belong to different footage; inspect the new source before rendering`);
    if (!Array.isArray(privacy.regions)) fail(`${key} privacy regions must be an explicit array`);
    for (const value of privacy.regions as unknown[]) {
      const region = record(value, `${key} privacy region`);
      const fields = ['fromFrame', 'untilFrame', 'x', 'y', 'width', 'height'] as const;
      for (const field of fields) {
        const n = number(region[field], `${key} privacy ${field}`);
        if (!Number.isInteger(n) || n < 0) fail(`${key} privacy ${field} must be a nonnegative integer`);
      }
      const {fromFrame, untilFrame, x, y, width, height} = region as NativePrivacyEdit['regions'][number];
      if (fromFrame >= untilFrame || untilFrame > Math.round(sourceDuration * (clip.sourceFps as number))) fail(`${key} privacy interval exceeds its source frames`);
      if (width === 0 || height === 0 || x + width > (clip.width as number) || y + height > (clip.height as number)) fail(`${key} privacy rectangle exceeds its source image`);
      if (width < 160 || height < 16) fail(`${key} privacy rectangle must fit its visible label without covering adjacent evidence`);
      if (region.label !== 'Host path redacted') fail(`${key} privacy edit must remain visibly labeled`);
    }
  }
}

export const parseApertureFilmProps = (value: unknown): ApertureFilmProps => {
  assertApertureEvidence(value);
  return value;
};
