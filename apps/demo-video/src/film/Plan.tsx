// One request becomes six tasks for five agents, which then run in parallel
// on the recorded clock. Local frames; see PytxoFilm for placement.
import {AbsoluteFill, interpolate, useCurrentFrame} from "remotion";
import {Caption, Check, Chip, Cursor, Logo, Window, fade, ledger, mix, mono, ramp, sans, T, useSettle, usePop, vendorOf} from "./kit";
import {cardBox, CARD_H, columnHeader, endOf, printed, RUN_SECONDS, startOf, TASK_TITLE, workers, type Box, type Worker} from "./layout";

export const PLAN_FRAMES = 1420;
const TYPE_FROM = 70, TYPE_TO = 300, BUILD = 352, SPLIT = 372, RUN_FROM = 600, RUN_TO = 1180, ORDER_FROM = 870, ORDER_TO = 1070;
const MISSION = workers.map((worker) => worker.request).join("\n");
const COMPOSER: Box = {x: 400, y: 270, w: 1120, h: 560};
const TEAM = ["codex", "claude", "cursor", "opencode", "agy"];
const SHARED = "src/components/filter-bar.js";

const Composer = () => {
  const frame = useCurrentFrame();
  const enter = useSettle(0, 48);
  const leave = ramp(frame, SPLIT, SPLIT + 40);
  const typed = MISSION.slice(0, Math.round(interpolate(frame, [TYPE_FROM, TYPE_TO], [0, MISSION.length], {extrapolateLeft: "clamp", extrapolateRight: "clamp"})));
  const caretOn = Math.floor(frame / 18) % 2 === 0 || (frame > TYPE_FROM && frame < TYPE_TO);
  const press = frame >= BUILD && frame < BUILD + 10 ? 0.96 : 1;
  return <Window style={{left: COMPOSER.x, top: COMPOSER.y + (1 - enter) * 60 - leave * 80, width: COMPOSER.w, height: COMPOSER.h, opacity: Math.min(enter, 1 - leave), transform: `scale(${1 - leave * 0.08})`}} title="New work">
    <div style={{padding: "26px 34px 0", fontFamily: sans}}>
      <div style={{fontSize: 18, color: T.muted, marginBottom: 12}}>What should Pytxo do?</div>
      <div style={{height: 300, fontSize: 19, lineHeight: 1.62, color: T.soft, whiteSpace: "pre-wrap", overflow: "hidden"}}>
        {typed}<span style={{display: "inline-block", width: 2, height: 24, marginLeft: 1, verticalAlign: -5, background: T.ink, opacity: caretOn ? 1 : 0}} />
      </div>
    </div>
    <div style={{position: "absolute", left: 34, right: 34, bottom: 26, display: "flex", alignItems: "center", gap: 14, fontFamily: sans}}>
      <span style={{fontSize: 17, color: T.muted, marginRight: 4}}>Agents</span>
      {TEAM.map((cli, index) => {
        const pop = usePop(TYPE_TO + 8 + index * 6);
        return <span key={cli} style={{display: "inline-flex", alignItems: "center", gap: 9, padding: "6px 12px 6px 6px", borderRadius: 10, background: T.hush, transform: `scale(${pop})`, opacity: Math.min(1, pop)}}>
          <Logo cli={cli} size={30} /><span style={{fontSize: 16, color: T.soft, fontWeight: 500}}>{vendorOf(cli)}</span>
        </span>;
      })}
      <span style={{marginLeft: "auto", padding: "12px 22px", borderRadius: 10, background: T.ink, color: "#fff", fontSize: 18, fontWeight: 600, transform: `scale(${press})`}}>Build plan</span>
    </div>
  </Window>;
};

const Status = ({worker, seconds}: {worker: Worker; seconds: number}) => {
  if (seconds < startOf(worker)) return <Chip tone="muted">Waiting for inputs</Chip>;
  if (seconds < endOf(worker)) {
    const dots = ".".repeat(1 + (Math.floor(seconds * 3) % 3));
    return <Chip tone="amber"><span style={{width: 7, height: 7, borderRadius: 4, background: T.amber}} />Working{dots}</Chip>;
  }
  return worker.filesPrepared.length
    ? <Chip tone="green"><Check size={16} />Checks passed</Chip>
    : <Chip tone="muted"><Check size={16} color={T.muted} />No changes</Chip>;
};

const TaskCard = ({worker, index}: {worker: Worker; index: number}) => {
  const frame = useCurrentFrame();
  const box = cardBox(worker);
  const fly = useSettle(SPLIT + 6 + index * 7, 54);
  const seconds = interpolate(frame, [RUN_FROM, RUN_TO], [0, RUN_SECONDS], {extrapolateLeft: "clamp", extrapolateRight: "clamp"});
  const running = frame >= RUN_FROM;
  const lines = printed(worker);
  const progress = Math.max(0, Math.min(1, (seconds - startOf(worker)) / Math.max(1, endOf(worker) - startOf(worker))));
  const shown = running ? lines.slice(0, Math.ceil(progress * lines.length)).slice(-2) : [];
  const ordering = fade(frame, ORDER_FROM, ORDER_TO, 20);
  const done = running && seconds >= endOf(worker);
  const origin = {x: COMPOSER.x + COMPOSER.w / 2 - box.w / 2, y: COMPOSER.y + COMPOSER.h / 2 - box.h / 2};
  return <div style={{position: "absolute", left: mix(origin.x, box.x, fly), top: mix(origin.y, box.y, fly), width: box.w, height: CARD_H, opacity: Math.min(1, fly * 2), transform: `scale(${mix(0.86, 1, fly)})`, background: T.card, borderRadius: 16, border: `1px solid ${done && worker.filesPrepared.length ? "#CFE6E0" : T.rule}`, boxShadow: "0 1px 2px rgba(25,25,24,.05), 0 8px 24px rgba(25,25,24,.06)", padding: "18px 20px", boxSizing: "border-box", fontFamily: sans}}>
    <div style={{display: "flex", alignItems: "center", gap: 14}}>
      <Logo cli={worker.cli} size={44} />
      <div style={{minWidth: 0}}>
        <div style={{fontSize: 20, fontWeight: 600, color: T.ink, letterSpacing: "-0.01em"}}>{TASK_TITLE[worker.taskId]}</div>
        <div style={{fontSize: 16, color: T.muted, marginTop: 2}}>{vendorOf(worker.cli)}</div>
      </div>
      <div style={{marginLeft: "auto"}}>{running ? <Status worker={worker} seconds={seconds} /> : <Chip tone="muted">Planned</Chip>}</div>
    </div>
    <div style={{height: 44, marginTop: 10, fontFamily: mono, fontSize: 15, lineHeight: "22px", color: T.muted, overflow: "hidden"}}>
      {shown.map((line, key) => <div key={key} style={{whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", color: line.startsWith("+") ? T.green : T.muted}}>{line}</div>)}
    </div>
    <div style={{display: "flex", gap: 8, flexWrap: "nowrap", overflow: "hidden"}}>
      {worker.paths.map((file) => {
        const shared = file === SHARED;
        return <span key={file} style={{fontFamily: mono, fontSize: 14, padding: "3px 8px", borderRadius: 6, whiteSpace: "nowrap", background: shared ? `rgba(183,121,31,${0.06 + ordering * 0.12})` : T.hush, color: shared && ordering > 0.1 ? T.amber : T.soft, boxShadow: shared ? `inset 0 0 0 ${ordering * 1.5}px ${T.amber}` : "none"}}>{file}</span>;
      })}
    </div>
  </div>;
};

/** Dependency lines: model before validation, and everything before the README. */
const Links = () => {
  const frame = useCurrentFrame();
  const draw = ramp(frame, SPLIT + 70, SPLIT + 150);
  const ordering = fade(frame, ORDER_FROM, ORDER_TO, 20);
  const byId = Object.fromEntries(workers.map((worker) => [worker.taskId, cardBox(worker)]));
  const curve = (from: Box, to: Box) => {
    const x1 = from.x + from.w, y1 = from.y + from.h / 2, x2 = to.x, y2 = to.y + to.h / 2;
    return `M${x1} ${y1} C${x1 + 26} ${y1}, ${x2 - 26} ${y2}, ${x2} ${y2}`;
  };
  const edges = workers.flatMap((worker) => worker.dependsOn.map((id) => ({from: byId[id], to: byId[worker.taskId], strong: worker.dependsOn.length === 1})));
  const shared = [byId["mission-3"], byId["mission-4"]];
  return <svg width={1920} height={1080} style={{position: "absolute", inset: 0}}>
    {edges.map((edge, index) => <path key={index} d={curve(edge.from, edge.to)} fill="none" stroke={edge.strong ? T.faint : T.rule} strokeWidth={2} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - draw} />)}
    <path d={curve(shared[0], shared[1])} fill="none" stroke={T.amber} strokeWidth={2.5} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - ordering} opacity={ordering} />
  </svg>;
};

const Clock = () => {
  const frame = useCurrentFrame();
  const seconds = interpolate(frame, [RUN_FROM, RUN_TO], [0, RUN_SECONDS], {extrapolateLeft: "clamp", extrapolateRight: "clamp"});
  const visible = fade(frame, RUN_FROM - 10, PLAN_FRAMES, 20);
  const clock = `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
  return <div style={{position: "absolute", right: 120, top: 104, textAlign: "right", opacity: visible, fontFamily: sans}}>
    <div style={{fontFamily: mono, fontSize: 44, fontWeight: 500, color: T.ink, fontVariantNumeric: "tabular-nums"}}>{clock}</div>
    <div style={{fontSize: 16, color: T.muted, marginTop: 4}}>recorded run · {Math.round(RUN_SECONDS / 60)} minutes, shown in {Math.round((RUN_TO - RUN_FROM) / 60)} s</div>
  </div>;
};

const Tally = () => {
  const frame = useCurrentFrame();
  const visible = fade(frame, RUN_TO + 20, PLAN_FRAMES + 30, 20);
  const pop = usePop(RUN_TO + 20);
  const files = workers.reduce((sum, worker) => sum + worker.filesPrepared.length, 0);
  return <div style={{position: "absolute", left: 0, right: 0, bottom: 34, display: "flex", justifyContent: "center", gap: 14, opacity: visible, transform: `translateY(${(1 - pop) * 16}px)`}}>
    <Chip tone="green" style={{fontSize: 19, padding: "8px 16px"}}><Check />{ledger.checks.passed} of {ledger.checks.recorded} task checks passed</Chip>
    <Chip style={{fontSize: 19, padding: "8px 16px"}}>{files} files prepared by {workers.filter((worker) => worker.filesPrepared.length).length} agents</Chip>
  </div>;
};

export const Plan = () => {
  const frame = useCurrentFrame();
  const headers = fade(frame, SPLIT + 40, PLAN_FRAMES + 30, 20);
  const waveLabel = ["Wave 1 · together", "Wave 2 · after its inputs", "Wave 3 · after everything"];
  return <AbsoluteFill style={{background: T.paper}}>
    <Caption step="01 · Describe the change once" text="One request." at={14} out={SPLIT + 10} />
    <Caption step="02 · Split by ownership" text="Six tasks. Five agents. Three waves." at={SPLIT + 24} out={RUN_FROM + 40} />
    <Caption step="03 · Every agent in its own copy" text="They work in parallel, never on your files." at={RUN_FROM + 56} out={ORDER_FROM} />
    <Caption step="04 · Shared files take turns" text="Two tasks own filter-bar.js. Pytxo orders them." at={ORDER_FROM + 14} out={RUN_TO + 10} />
    <Caption step="05 · Pytxo runs the checks" text="Each task is checked before review." at={RUN_TO + 24} out={PLAN_FRAMES + 30} />
    <Composer />
    {[0, 1, 2].map((wave) => <div key={wave} style={{position: "absolute", ...columnHeader(wave), fontFamily: sans, fontSize: 17, fontWeight: 500, color: T.muted, opacity: headers}}>{waveLabel[wave]}</div>)}
    <Links />
    {workers.map((worker, index) => <TaskCard key={worker.taskId} worker={worker} index={index} />)}
    <Clock />
    <Tally />
    <Cursor visibleFrom={34} visibleTo={SPLIT + 10} keys={[
      {at: 34, x: 1240, y: 900},
      {at: 62, x: 760, y: 420, click: true},
      {at: 300, x: 760, y: 430},
      {at: BUILD, x: 1440, y: 778, click: true},
      {at: SPLIT + 10, x: 1460, y: 800},
    ]} />
  </AbsoluteFill>;
};
