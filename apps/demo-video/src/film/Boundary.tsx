// The prepared files converge into one change set, the stale Apply is refused,
// the review is refreshed and the exact package is applied. Local frames.
import {AbsoluteFill, Img, interpolate, staticFile, useCurrentFrame} from "remotion";
import {Caption, Check, Chip, Cursor, Logo, Window, changes, fade, ledger, mix, mono, ramp, sans, short, T, useSettle} from "./kit";
import {cardBox, CARD_H, TASK_TITLE, workers} from "./layout";

export const BOUNDARY_FRAMES = 1200;
const SWITCH = 330, NOTE = 470, ATTEMPT = 530, REFRESH = 650, APPLY = 800, RESULT = 990;
const WIN = {x: 200, y: 250, w: 1520, h: 740};
const LIST = {x: WIN.x + 24, y: WIN.y + 48 + 84, row: 58, w: 440};
const files = changes.files;
const added = files.reduce((sum, file) => sum + file.added, 0);
const removed = files.reduce((sum, file) => sum + file.removed, 0);
const rowOf = (path: string) => files.findIndex((file) => file.path === path);
const firstFile = rowOf("src/model.mjs");
const secondFile = rowOf("src/theme.js");

/** The task cards from the run, fading as their files leave them. */
const Ghosts = () => {
  const frame = useCurrentFrame();
  const out = 1 - ramp(frame, 10, 60);
  return <>{workers.map((worker) => {
    const box = cardBox(worker);
    return <div key={worker.taskId} style={{position: "absolute", left: box.x, top: box.y, width: box.w, height: CARD_H, borderRadius: 16, background: T.card, border: `1px solid ${T.rule}`, opacity: out * 0.9, transform: `scale(${mix(0.96, 1, out)})`, padding: "18px 20px", boxSizing: "border-box", display: "flex", gap: 14, fontFamily: sans}}>
      <Logo cli={worker.cli} size={44} />
      <div style={{fontSize: 20, fontWeight: 600, color: T.ink}}>{TASK_TITLE[worker.taskId]}</div>
    </div>;
  })}</>;
};

const FileRow = ({file, index}: {file: (typeof files)[number]; index: number}) => {
  const frame = useCurrentFrame();
  const worker = workers.find((candidate) => candidate.taskId === file.task)!;
  const card = cardBox(worker);
  const fly = useSettle(8 + index * 6, 60);
  const selected = (frame < SWITCH ? firstFile : secondFile) === index;
  const applied = frame >= APPLY + 30 + index * 7;
  const tick = useSettle(APPLY + 30 + index * 7, 30);
  const x = mix(card.x + 20, LIST.x, fly), y = mix(card.y + CARD_H - 44, LIST.y + index * LIST.row, fly);
  return <div style={{position: "absolute", left: x, top: y, width: LIST.w, height: LIST.row - 8, borderRadius: 10, display: "flex", alignItems: "center", gap: 12, padding: "0 14px", boxSizing: "border-box", background: selected ? T.hush : "transparent", zIndex: 5}}>
    <Logo cli={worker.cli} size={26} />
    <span style={{fontFamily: mono, fontSize: 17, color: T.ink, whiteSpace: "nowrap"}}>{file.path}</span>
    <span style={{marginLeft: "auto", fontFamily: mono, fontSize: 16, whiteSpace: "nowrap", opacity: fly}}>
      {applied
        ? <span style={{display: "inline-flex", transform: `scale(${tick})`}}><Check size={20} /></span>
        : <><span style={{color: T.green}}>+{file.added}</span> <span style={{color: T.coral}}>−{file.removed}</span></>}
    </span>
  </div>;
};

const Diff = () => {
  const frame = useCurrentFrame();
  const index = frame < SWITCH ? firstFile : secondFile;
  const file = files[index];
  const since = frame < SWITCH ? 120 : SWITCH;
  const swap = ramp(frame, since, since + 20);
  const lines = file.lines.filter((line) => line.kind !== "hunk").slice(0, 26);
  const scroll = interpolate(frame, [since + 40, since + 200], [0, Math.max(0, lines.length - 13) * 30], {extrapolateLeft: "clamp", extrapolateRight: "clamp"});
  const left = LIST.x + LIST.w + 24;
  return <div style={{position: "absolute", left, top: WIN.y + 48 + 84, width: WIN.x + WIN.w - left - 24, height: 420, borderRadius: 12, background: T.code, border: `1px solid ${T.rule}`, overflow: "hidden", opacity: Math.min(ramp(frame, 90, 130), swap)}}>
    <div style={{height: 46, display: "flex", alignItems: "center", gap: 12, padding: "0 18px", borderBottom: `1px solid ${T.rule}`, fontFamily: sans, fontSize: 17, color: T.muted}}>
      <span style={{fontFamily: mono, color: T.ink}}>{file.path}</span>
      <span>prepared by {workers.find((worker) => worker.taskId === file.task)?.vendor}</span>
    </div>
    <div style={{transform: `translateY(${-scroll}px)`, paddingTop: 8}}>
      {lines.map((line, key) => {
        const tone = line.kind === "add" ? {bg: "#EDF6F3", fg: "#135E52", mark: "+"} : line.kind === "del" ? {bg: "#FBEFEC", fg: "#9B3A24", mark: "−"} : {bg: "transparent", fg: T.soft, mark: " "};
        return <div key={key} style={{display: "flex", height: 30, alignItems: "center", background: tone.bg, fontFamily: mono, fontSize: 16, color: tone.fg, whiteSpace: "pre"}}>
          <span style={{width: 52, textAlign: "right", paddingRight: 14, color: T.faint}}>{line.after ?? line.before}</span>
          <span style={{width: 20}}>{tone.mark}</span>
          <span style={{overflow: "hidden", textOverflow: "ellipsis"}}>{line.text}</span>
        </div>;
      })}
    </div>
  </div>;
};

/** Hex characters settle left to right from the refused digest to the refreshed one. */
const Digest = () => {
  const frame = useCurrentFrame();
  const from = short(ledger.run.staleDigest ?? ledger.run.packageDigest), to = short(ledger.run.packageDigest);
  const t = ramp(frame, REFRESH + 20, REFRESH + 60);
  const hex = "0123456789abcdef";
  const text = [...to].map((char, index) => {
    if (char === "…") return char;
    const settle = index / to.length;
    if (t >= settle + 0.15) return char;
    if (t <= settle) return from[index];
    return hex[(frame * 7 + index * 3) % 16];
  }).join("");
  return <span style={{fontFamily: mono, fontSize: 17, color: T.soft, padding: "6px 12px", borderRadius: 8, background: T.hush}}>{text}</span>;
};

const Footer = () => {
  const frame = useCurrentFrame();
  const refused = frame >= ATTEMPT + 18 && frame < REFRESH + 24;
  const rechecked = frame >= REFRESH + 24 && frame < APPLY + 20;
  const applied = frame >= APPLY + 20;
  const banner = useSettle(ATTEMPT + 18, 30);
  const tone = refused ? {bg: T.coralWash, fg: T.coral} : rechecked || applied ? {bg: T.greenWash, fg: T.green} : {bg: T.card, fg: T.muted};
  const message = applied ? `Applied · ${files.length} files written exactly as reviewed`
    : rechecked ? "Rechecked against the project as it is now"
    : refused ? "Apply refused · the project changed after review. Nothing was written."
    : "Not applied yet";
  const button = applied ? null : refused ? "Refresh review" : rechecked ? "Apply exact package" : "Apply reviewed changes";
  const pressAt = refused ? REFRESH : rechecked ? APPLY : ATTEMPT;
  const press = frame >= pressAt && frame < pressAt + 10 ? 0.96 : 1;
  return <div style={{position: "absolute", left: WIN.x + 1, width: WIN.w - 2, top: WIN.y + WIN.h - 92, height: 91, borderTop: `1px solid ${T.rule}`, background: tone.bg, display: "flex", alignItems: "center", padding: "0 28px", boxSizing: "border-box", gap: 14, fontFamily: sans, borderRadius: "0 0 18px 18px"}}>
    {(refused || rechecked || applied) && <span style={{transform: `scale(${refused ? banner : 1})`}}>{refused ? <svg width="22" height="22" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="none" stroke={T.coral} strokeWidth="1.6" /><path d="M8 4.5v4.2M8 11.2v.3" stroke={T.coral} strokeWidth="1.8" strokeLinecap="round" /></svg> : <Check size={22} />}</span>}
    <span style={{fontSize: 20, fontWeight: refused || applied ? 600 : 500, color: tone.fg}}>{message}</span>
    {button && <span style={{marginLeft: "auto", padding: "13px 24px", borderRadius: 10, background: refused ? T.card : T.ink, color: refused ? T.ink : "#fff", border: refused ? `1px solid ${T.rule}` : "none", fontSize: 18, fontWeight: 600, transform: `scale(${press})`}}>{button}</span>}
  </div>;
};

const Note = () => {
  const frame = useCurrentFrame();
  const enter = useSettle(NOTE, 40);
  const out = ramp(frame, REFRESH - 10, REFRESH + 20);
  return <div style={{position: "absolute", left: 1330, top: 150 - (1 - enter) * 30, width: 390, opacity: Math.min(enter, 1 - out), background: T.card, borderRadius: 14, boxShadow: "0 8px 24px rgba(25,25,24,.10)", border: `1px solid ${T.rule}`, padding: "14px 18px", fontFamily: sans, display: "flex", gap: 12, alignItems: "center"}}>
    <span style={{width: 34, height: 40, borderRadius: 6, background: T.hush, border: `1px solid ${T.rule}`}} />
    <div><div style={{fontFamily: mono, fontSize: 17, color: T.ink}}>operator-note.txt</div><div style={{fontSize: 15, color: T.muted, marginTop: 2}}>added to the project after review</div></div>
  </div>;
};

export const Boundary = () => {
  const frame = useCurrentFrame();
  const open = useSettle(30, 50);
  const toResult = useSettle(RESULT, 60);
  const result = useSettle(RESULT + 16, 60);
  return <AbsoluteFill style={{background: T.paper}}>
    <Ghosts />
    <Caption step="06 · Review one change set" text="Every changed line, before anything is written." at={24} out={NOTE} />
    <Caption step="07 · The boundary holds" text="The project moved, so Apply refused." at={NOTE + 16} out={APPLY - 30} />
    <Caption step="08 · Apply what you reviewed" text="Exactly these seven files. Nothing else." at={APPLY - 14} out={RESULT + 10} />
    <Caption step="09 · The result" text="The task board, applied and running." at={RESULT + 24} out={BOUNDARY_FRAMES + 30} />
    <div style={{position: "absolute", inset: 0, transform: `translate(${-toResult * 330}px, ${toResult * 60}px) scale(${1 - toResult * 0.34})`, transformOrigin: "200px 620px", opacity: 1 - ramp(frame, RESULT + 120, RESULT + 170) * 0.6}}>
      <Window style={{left: WIN.x, top: WIN.y + (1 - open) * 50, width: WIN.w, height: WIN.h, opacity: open}} title="Review changes">
        <div style={{height: 84, display: "flex", alignItems: "center", gap: 12, padding: "0 28px", fontFamily: sans, borderBottom: `1px solid ${T.rule}`}}>
          <span style={{fontSize: 24, fontWeight: 600, color: T.ink, letterSpacing: "-0.02em", marginRight: 6}}>Change set</span>
          <Chip>{files.length} files</Chip>
          <Chip><span style={{color: T.green}}>+{added}</span><span style={{color: T.coral}}>−{removed}</span></Chip>
          <Chip tone="green"><Check size={16} />{ledger.checks.passed}/{ledger.checks.recorded} checks passed</Chip>
          <span style={{marginLeft: "auto"}}><Digest /></span>
        </div>
      </Window>
      <div style={{opacity: open}}><Diff /></div>
      {files.map((file, index) => <FileRow key={file.path} file={file} index={index} />)}
      <div style={{opacity: open}}><Footer /></div>
    </div>
    <Note />
    <Window title="localhost · Task board" style={{left: mix(1980, 920, result), top: 250, width: 880, height: 600, opacity: result}}>
      <Img src={staticFile("fleet/result.png")} style={{width: 880, height: 550, objectFit: "cover", objectPosition: "50% 0%"}} />
    </Window>
    <div style={{position: "absolute", left: 920, top: 880, opacity: fade(frame, RESULT + 60, BOUNDARY_FRAMES + 30), display: "flex", gap: 12}}>
      <Chip tone="green" style={{fontSize: 19, padding: "8px 16px"}}><Check />npm test · 10 of 10 passed</Chip>
      <Chip style={{fontSize: 19, padding: "8px 16px"}}>English · Español</Chip>
    </div>
    <Cursor visibleFrom={140} visibleTo={RESULT + 20} keys={[
      {at: 140, x: 1500, y: 960},
      {at: 300, x: LIST.x + 200, y: LIST.y + secondFile * LIST.row + 24},
      {at: SWITCH, x: LIST.x + 200, y: LIST.y + secondFile * LIST.row + 24, click: true},
      {at: ATTEMPT, x: 1570, y: WIN.y + WIN.h - 46, click: true},
      {at: REFRESH, x: 1600, y: WIN.y + WIN.h - 46, click: true},
      {at: APPLY, x: 1580, y: WIN.y + WIN.h - 46, click: true},
      {at: RESULT, x: 1620, y: 900},
    ]} />
  </AbsoluteFill>;
};
