"use client";

import { useEffect, useRef } from "react";

import { Mark } from "@/components/site/mark";
import s from "@/components/site/site.module.css";

// The mark, in ASCII: two strokes that meet at one point, and that point is the signal.
// Pass 1 raymarches the mark once per character cell (2x2 rays) into a cols x rows texture. Pass 2 draws each
// device pixel from its cell's glyph, read texel-exact from an IBM Plex Mono atlas drawn at runtime.
// The geometry is fitted to the logo: one unit is the strokes' 440 px height, centred on (256.5, 256).
const RAMP = " .:-=+*x#%@";
const VERT = `#version 300 es
void main() { vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2)); gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0); }`;
const MARCH = `#version 300 es
precision highp float;
uniform vec3 uMark;
uniform vec2 uCell;
uniform mat3 uInv;
uniform vec3 uLight;
uniform vec3 uRight;
uniform vec3 uOff1;
uniform vec3 uOff2;
uniform float uDot;
uniform float uHole;
uniform float uScale;
uniform float uFlare;
uniform vec4 uVeil[8];
uniform float uFloor[8];
uniform int uVeils;
uniform vec3 uTrail;
uniform vec4 uSoft;
out vec4 outCell;
const float CAM = 4.0;
const vec2 N1 = vec2(0.894427, 0.447214);
const vec2 N2 = vec2(0.894427, -0.447214);
const vec2 C2 = vec2(-0.215, -0.5);
const vec3 DOT = vec3(0.05, -0.1, 0.0);
const float W = 0.123;
const float DEPTH = 0.085;
// One stroke: a bar with a flat face and round sides, cut flat at y0 and y1 like the logo.
float bar(vec3 p, vec2 n, vec2 c, float y0, float y1) {
  vec2 q = abs(vec2(dot(p.xy - c, n), p.z)) - vec2(W - DEPTH, 0.0);
  return max(length(max(q, 0.0)) - DEPTH, max(y0 - p.y, p.y - y1));
}
float scene(vec3 p, out float sphere) {
  float s1 = bar(p - uOff1, N1, vec2(0.0), -0.5, 0.5);
  float s2 = bar(p - uOff2, N2, C2, -0.5, DOT.y);
  // The knockout: a hole through both strokes around the dot, open only while the dot is.
  float strokes = max(min(s1, s2), uHole - length(p.xy - DOT.xy));
  float ball = uDot > 0.002 ? length(p - DOT) - uDot : 1e3;
  sphere = step(ball, strokes);
  return min(strokes, ball);
}
vec3 normalAt(vec3 p) {
  const vec2 k = vec2(1.0, -1.0);
  const float e = 0.0015;
  float m;
  return normalize(k.xyy * scene(p + k.xyy * e, m) + k.yyx * scene(p + k.yyx * e, m) +
                   k.yxy * scene(p + k.yxy * e, m) + k.xxx * scene(p + k.xxx * e, m));
}
float occlusion(vec3 p, vec3 n) {
  float o = 0.0, w = 1.0, m;
  for (int i = 1; i <= 3; i++) {
    float h = 0.03 * float(i);
    o += (h - scene(p + n * h, m)) * w;
    w *= 0.6;
  }
  return clamp(1.0 - 4.0 * o, 0.0, 1.0);
}
void main() {
  vec2 base = floor(gl_FragCoord.xy) * uCell;
  vec2 mid = base + 0.5 * uCell;
  // Dim the cells under the copy (less under the display type, more under small text) so text stays readable,
  // and fade at a stage edge that meets another section.
  float veil = 1.0;
  for (int i = 0; i < 8; i++) {
    if (i >= uVeils) break;
    vec2 c = 0.5 * (uVeil[i].xy + uVeil[i].zw);
    vec2 q = abs(mid - c) - 0.5 * (uVeil[i].zw - uVeil[i].xy);
    veil = min(veil, mix(uFloor[i], 1.0, smoothstep(0.0, uSoft.x, length(max(q, 0.0)) + min(max(q.x, q.y), 0.0))));
  }
  veil *= smoothstep(0.0, uSoft.y, mid.y) * smoothstep(0.0, uSoft.z, uSoft.w - mid.y);
  vec3 ro = uInv * vec3(0.0, 0.0, CAM) / uScale;
  float c = dot(ro, ro) - 1.0;
  float shade = 0.0, hits = 0.0, lit = 0.0, across = 0.0;
  for (int i = 0; i < 4; i++) {
    vec2 q = base + (vec2(float(i & 1), float(i >> 1)) * 0.5 + 0.25) * uCell;
    vec3 rd = uInv * normalize(vec3((q - uMark.xy) / uMark.z, -CAM));
    float b = dot(ro, rd);
    float h = b * b - c;
    if (h < 0.0) continue;
    h = sqrt(h);
    float t = max(-b - h, 0.0), tEnd = -b + h, m = 0.0;
    bool hit = false;
    for (int k = 0; k < 64; k++) {
      float d = scene(ro + rd * t, m);
      if (d < 0.0015) { hit = true; break; }
      t += d * 0.9;
      if (t > tEnd) break;
    }
    if (!hit) {
      // A smear where the short stroke just was, so the snap reads even in a still frame.
      if (uTrail.z > 0.0 && rd.z < 0.0) {
        vec2 p0 = (ro + rd * (-ro.z / rd.z)).xy;
        for (int j = 0; j < 4; j++) {
          vec2 o = p0 - mix(uTrail.xy, uOff2.xy, float(j) / 3.0);
          if (max(abs(dot(o - C2, N2)) - W, max(-0.5 - o.y, o.y - DOT.y)) < 0.0) { shade += 0.32 * uTrail.z; break; }
        }
      }
      continue;
    }
    vec3 p = ro + rd * t;
    vec3 n = normalAt(p);
    float diffuse = max(dot(n, uLight), 0.0);
    float spec = pow(max(dot(n, normalize(uLight - rd)), 0.0), 36.0);
    float s;
    if (m > 0.5) {
      s = 0.24 + 0.76 * diffuse + 0.55 * spec + 0.9 * uFlare;
      lit += 1.0;
      across += clamp(0.5 + 0.5 * dot(p - DOT, uRight) / uDot, 0.0, 1.0);
    } else {
      s = (0.06 + diffuse) * occlusion(p, n) + 0.4 * spec;
    }
    shade += clamp(s, 0.0, 1.0);
    hits += 1.0;
  }
  outCell = vec4(shade * 0.25 * mix(0.6, 1.0, veil), hits > 0.0 ? lit / hits : 0.0, lit > 0.0 ? across / lit : 0.0, veil);
}`;
const GLYPH = `#version 300 es
precision highp float;
precision highp int;
uniform sampler2D uCells;
uniform sampler2D uAtlas;
uniform ivec2 uCellPx;
out vec4 outColor;
const vec3 INK = vec3(0.0314, 0.0314, 0.0392);
const vec3 PAPER = vec3(0.9255, 0.9176, 0.8941);
const vec3 MAG = vec3(0.9412, 0.302, 0.6392);
const vec3 COR = vec3(1.0, 0.4157, 0.2902);
void main() {
  ivec2 px = ivec2(gl_FragCoord.xy);
  ivec2 cell = px / uCellPx;
  vec4 c = texelFetch(uCells, cell, 0);
  int glyph = int(c.r * 10.999);
  if (glyph == 0) { outColor = vec4(INK, 1.0); return; }
  ivec2 local = px - cell * uCellPx;
  float ink = texelFetch(uAtlas, ivec2(glyph * uCellPx.x + local.x, uCellPx.y - 1 - local.y), 0).r;
  vec3 tone = c.g > 0.5 ? mix(MAG, COR, c.b) : mix(INK, PAPER, 0.45 + 0.55 * smoothstep(0.05, 0.7, c.r));
  outColor = vec4(mix(INK, tone, ink * c.a), 1.0);
}`;
const UNIFORMS = ["uMark", "uCell", "uInv", "uLight", "uRight", "uOff1", "uOff2", "uDot", "uHole", "uScale", "uFlare",
  "uVeil", "uFloor", "uVeils", "uTrail", "uSoft"] as const;

// The story in time. The strokes start apart in plain paper and pull together, faster as they close; at SNAP they
// meet and the dot flares in the signal. Then a slow sway, and every PERIOD seconds a short part-and-snap beat.
// With reduced motion the mark is drawn once, at REST.
const SNAP = 1.15, PERIOD = 10, REST = 5;
const FONT = '500 16px "IBM Plex Mono"';
const LX = -0.451, LY = 0.551, LZ = 0.702;
const smooth = (a: number, b: number, x: number) => { const k = Math.min(Math.max((x - a) / (b - a), 0), 1); return k * k * (3 - 2 * k); };
const clamp01 = (x: number) => Math.min(Math.max(x, 0), 1);

/** Left, top, right and bottom in CSS px from the field's top left, then how far to dim the field under it. */
type Keep = [number, number, number, number, number];

/** The hero's backdrop. The copy above it marks what to keep readable with `data-keep="text|box:floor"`. */
export function MarkField() {
  const field = useRef<HTMLDivElement>(null);
  useEffect(() => start(field.current!), []);
  return (
    <div
      ref={field}
      className={s.field}
      role="img"
      aria-label="The Pytxo mark drawn in text characters. Its two strokes start apart, snap together, and the point where they meet lights up in magenta to coral."
    >
      <Mark id="still" className={s.still} />
    </div>
  );
}

function start(field: HTMLDivElement): () => void {
  const stage = field.parentElement!;
  const still = field.querySelector("svg")!;
  const motion = matchMedia("(prefers-reduced-motion: reduce)");
  let reduce = motion.matches;
  // Each entry: an element, whether to measure its text rather than its box, and how far to dim the field under it.
  // In page order: the headline's two sentences, the pitch, the command, the link.
  const keepEls = [...stage.querySelectorAll<HTMLElement>("[data-keep]")].map((el) => {
    const [kind, floor] = el.dataset.keep!.split(":");
    return { el, text: kind === "text", floor: Number(floor) };
  });
  const undo: (() => void)[] = [];
  let dead = false, raf = 0;

  const textRect = (el: Element) => { const r = document.createRange(); r.selectNodeContents(el); return r.getBoundingClientRect(); };
  const keepOf = (f: DOMRect) => keepEls.map(({ el, text, floor }): Keep => {
    const r = text ? textRect(el) : el.getBoundingClientRect();
    return [r.left - f.left, r.top - f.top, r.right - f.left, r.bottom - f.top, floor];
  });
  // Where the mark goes, in CSS px from the field's top left: the dot's centre (x, y), px per logo unit (k), the copy
  // to dim the field under (keep), and [top, bottom] fades where the field meets the header and the next section.
  const place = (f: DOMRect, W: number, H: number) => {
    const keep = keepOf(f), a = keep[0], b = keep[1], right = Math.max(a[2], b[2]), y = (a[3] + b[1]) / 2;
    // Landscape: the dot sits level with the seam between the headline's two sentences, and the mark fills the room
    // right of the headline: its right edge (148.5 logo px right of the dot's centre) on the field's edge, the dot
    // (radius 66) 48 px clear of the type and below the top fade, and the dot at most a quarter of the height across.
    // The long stroke comes down from under the header and the short one runs down beside the copy: its left edge
    // leaves the dot's level 89.1 logo px left of the centre at slope 2, and stays 48 px clear of the pitch and the link.
    const clear = (e: Keep) => (W - e[2] - 48 - ((e[1] + e[3]) / 2 - y) / 2) / 237.6;
    const k = Math.min((W - right - 48) / 214.5, (H * 0.24) / 66, (y - 56) / 66, clear(keep[2]), clear(keep[4]));
    if (W >= 768 && W > H && k * 66 >= 90) return { k, x: W - 148.5 * k, y, keep, fade: [48, 140] };
    // Portrait and phones: the dot sits above the headline, and its legs run behind it, dimmed across the full width.
    for (const e of keep.slice(0, 2)) { e[0] = 0; e[2] = W; }
    const r = Math.max(Math.min(a[1] / 2 - 10, W * 0.2), 40);
    return { k: r / 66, x: W - 1.6 * r, y: a[1] - r - 12, keep, fade: [48, 140] };
  };

  // With WebGL2 missing or the context lost, the logo stands in at the same place and size, its strokes faint.
  const placeStill = () => {
    const f = field.getBoundingClientRect(), g = place(f, f.width, f.height);
    still.style.setProperty("--s", `${512 * g.k}px`);
    still.style.setProperty("--x", `${g.x - 278.5 * g.k}px`);
    still.style.setProperty("--y", `${g.y - 300 * g.k}px`);
  };
  // The still follows the field and the copy as they resize or reflow, as the mark would.
  let follow: ResizeObserver | undefined;
  const fail = () => {
    field.classList.add(s.noGl);
    placeStill();
    if (follow) return;
    follow = new ResizeObserver(placeStill);
    follow.observe(field);
    for (const { el } of keepEls) follow.observe(el);
    undo.push(() => follow!.disconnect());
  };

  // A fresh canvas per mount, so a remount never inherits a lost context.
  const canvas = document.createElement("canvas");
  canvas.setAttribute("aria-hidden", "true");
  field.prepend(canvas);
  undo.push(() => { canvas.remove(); field.classList.remove(s.on, s.noGl); });

  const run = async () => {
    const gl = canvas.getContext("webgl2", { alpha: false, antialias: false, depth: false, stencil: false });
    if (!gl) return fail();
    let ok = false, seen = false, shown = false, last = 0, t = reduce ? REST : 0;
    let tiltX = 0, tiltY = 0, aimX = 0, aimY = 0, pointerOn = false;
    let cssW = 1, cssH = 1, W = 1, H = 1, dpr = 1, cw = 0, ch = 0, cols = 1, rows = 1;
    const P = { apart: 1, dot: 0, flare: 0, kick: 0 };
    const rot = new Float32Array(9), veil = new Float32Array(32), floor = new Float32Array(8);
    const atlas = document.createElement("canvas"), pen = atlas.getContext("2d")!;
    canvas.addEventListener("webglcontextlost", (e) => { e.preventDefault(); ok = false; if (!dead) fail(); });
    undo.push(() => gl.getExtension("WEBGL_lose_context")?.loseContext());

    const pose = (at: number) => {
      if (at < SNAP) { const u = at / SNAP; P.apart = 1 - u * u * u; P.dot = P.flare = P.kick = 0; return; }
      const v = (at - SNAP) % PERIOD, w = PERIOD - v, u = Math.max(1 - w / 0.9, 0);
      P.apart = 0.55 * smooth(2.6, 1.6, w) * (1 - u * u * u);
      P.dot = (1 - Math.exp(-v / 0.05)) * (1 + 0.18 * Math.exp(-v / 0.35)) * smooth(2.2, 2.6, w);
      P.flare = Math.exp(-v / 0.35);
      P.kick = (v / 0.07) * Math.exp(1 - v / 0.07);
    };

    // Link without blocking the main thread: with KHR_parallel_shader_compile the driver compiles in the background
    // and we look in once a frame before reading LINK_STATUS.
    const ext = gl.getExtension("KHR_parallel_shader_compile");
    const [march, glyph] = [MARCH, GLYPH].map((src) => {
      const p = gl.createProgram();
      for (const [type, text] of [[gl.VERTEX_SHADER, VERT], [gl.FRAGMENT_SHADER, src]] as const) {
        const sh = gl.createShader(type)!;
        gl.shaderSource(sh, text);
        gl.compileShader(sh);
        gl.attachShader(p, sh);
      }
      gl.linkProgram(p);
      return p;
    });
    const linked = (status: number) => [march, glyph].every((p) => gl.getProgramParameter(p, status));
    while (ext && !dead && !gl.isContextLost() && !linked(ext.COMPLETION_STATUS_KHR)) await new Promise(requestAnimationFrame);
    if (dead) return;
    if (gl.isContextLost() || !linked(gl.LINK_STATUS)) return fail();
    const U = Object.fromEntries(UNIFORMS.map((n) => [n, gl.getUniformLocation(march, n)])) as Record<(typeof UNIFORMS)[number], WebGLUniformLocation | null>;
    const cellPx = gl.getUniformLocation(glyph, "uCellPx");
    gl.useProgram(glyph);
    gl.uniform1i(gl.getUniformLocation(glyph, "uCells"), 0);
    gl.uniform1i(gl.getUniformLocation(glyph, "uAtlas"), 1);
    gl.bindVertexArray(gl.createVertexArray());
    const texture = (unit: number) => {
      const tex = gl.createTexture();
      gl.activeTexture(unit);
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      return tex;
    };
    const cells = texture(gl.TEXTURE0);
    const atlasTex = texture(gl.TEXTURE1);
    const fbo = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, cells, 0);

    const drawAtlas = () => {
      atlas.width = cw * RAMP.length;
      atlas.height = ch;
      pen.fillStyle = "#000";
      pen.fillRect(0, 0, atlas.width, ch);
      pen.fillStyle = "#fff";
      pen.textAlign = "center";
      pen.textBaseline = "alphabetic";
      pen.font = `500 ${cw / 0.6}px "IBM Plex Mono", ui-monospace, monospace`;
      for (let i = 1; i < RAMP.length; i++) pen.fillText(RAMP[i], (i + 0.5) * cw, Math.round(ch * 0.76));
      gl.activeTexture(gl.TEXTURE1);
      gl.bindTexture(gl.TEXTURE_2D, atlasTex);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, atlas);
    };
    const layout = () => {
      const g = place(field.getBoundingClientRect(), cssW, cssH), d = dpr, n = Math.min(g.keep.length, 8);
      for (let i = 0; i < n; i++) {
        const r = g.keep[i];
        veil[4 * i] = (r[0] - 8) * d; veil[4 * i + 1] = (cssH - r[3] - 8) * d; veil[4 * i + 2] = (r[2] + 8) * d; veil[4 * i + 3] = (cssH - r[1] + 8) * d;
        floor[i] = r[4];
      }
      gl.useProgram(march);
      gl.uniform4fv(U.uVeil, veil);
      gl.uniform1fv(U.uFloor, floor);
      gl.uniform1i(U.uVeils, n);
      gl.uniform4f(U.uSoft, 22 * d, Math.max(g.fade[1], 1) * d, Math.max(g.fade[0], 1) * d, H);
      gl.uniform3f(U.uMark, (g.x - 22 * g.k) * d, (cssH - g.y + 44 * g.k) * d, 440 * g.k * d);
    };
    // The glyph pass runs per device pixel, up to 3x; the raymarch stays on the cell grid whatever the DPR.
    const size = (e: ResizeObserverEntry) => {
      const css = e.contentBoxSize?.[0] ?? { inlineSize: e.contentRect.width, blockSize: e.contentRect.height };
      const ratio = window.devicePixelRatio || 1, dev = ratio <= 3 ? e.devicePixelContentBoxSize?.[0] : undefined;
      cssW = Math.max(1, css.inlineSize);
      cssH = Math.max(1, css.blockSize);
      W = Math.max(1, dev ? dev.inlineSize : Math.round(cssW * Math.min(ratio, 3)));
      H = Math.max(1, dev ? dev.blockSize : Math.round(cssH * Math.min(ratio, 3)));
      if (canvas.width !== W || canvas.height !== H) { canvas.width = W; canvas.height = H; }
      dpr = W / cssW;
      const nw = Math.max(5, Math.round(9 * dpr)), nh = Math.round(nw * 1.75);
      if (nw !== cw || nh !== ch) { cw = nw; ch = nh; drawAtlas(); gl.useProgram(glyph); gl.uniform2i(cellPx, cw, ch); }
      cols = Math.ceil(W / cw);
      rows = Math.ceil(H / ch);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, cells);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, cols, rows, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
      gl.useProgram(march);
      gl.uniform2f(U.uCell, cw, ch);
      layout();
    };
    const render = () => {
      // Where the short stroke was a moment ago, for the smear; only a fast move (the snap) leaves one.
      pose(t - 0.2);
      const was = P.apart;
      pose(t);
      const yaw = -0.08 + 0.15 * Math.sin(0.21 * t + 0.6) + 0.1 * tiltX;
      const pitch = 0.06 + 0.035 * Math.sin(0.17 * t) + 0.1 * tiltY;
      const cy = Math.cos(yaw), sy = Math.sin(yaw), cx = Math.cos(pitch), sx = Math.sin(pitch), a = P.apart, r = rot;
      // Object to view is Rx(pitch) * Ry(yaw); uInv is its transpose, column-major. The key light stays upper left in view.
      r[0] = cy; r[1] = 0; r[2] = sy;
      r[3] = sx * sy; r[4] = cx; r[5] = -sx * cy;
      r[6] = -cx * sy; r[7] = sx; r[8] = cx * cy;
      gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
      gl.viewport(0, 0, cols, rows);
      gl.useProgram(march);
      gl.uniformMatrix3fv(U.uInv, false, r);
      gl.uniform3f(U.uLight, r[0] * LX + r[3] * LY + r[6] * LZ, r[1] * LX + r[4] * LY + r[7] * LZ, r[2] * LX + r[5] * LY + r[8] * LZ);
      gl.uniform3f(U.uRight, cy, 0, sy);
      gl.uniform3f(U.uOff1, 0.06 * a, 0.03 * a, -0.1 * a);
      // The short stroke parts down and to the left, clear of the long one but not across the copy.
      gl.uniform3f(U.uOff2, -0.15 * a, -0.2 * a, 0.1 * a);
      gl.uniform3f(U.uTrail, -0.15 * was, -0.2 * was, clamp01((0.25 * Math.abs(was - a) - 0.03) / 0.03));
      gl.uniform1f(U.uDot, 0.15 * P.dot);
      gl.uniform1f(U.uHole, 0.2125 * P.dot);
      gl.uniform1f(U.uScale, 1 + 0.008 * Math.sin(1.05 * t) + 0.018 * P.kick);
      gl.uniform1f(U.uFlare, P.flare);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      gl.viewport(0, 0, W, H);
      gl.useProgram(glyph);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      if (!shown) { shown = true; requestAnimationFrame(() => field.classList.add(s.on)); }
    };

    const loop = (now: number) => {
      raf = 0;
      const dt = Math.min(Math.max((now - last) / 1000, 0), 0.05);
      last = now;
      const ease = 1 - Math.exp(-dt * 2.5);
      tiltX += ((pointerOn ? aimX : 0) - tiltX) * ease;
      tiltY += ((pointerOn ? aimY : 0) - tiltY) * ease;
      if (ok && seen && !reduce && !document.hidden) { t += dt; render(); raf = requestAnimationFrame(loop); }
    };
    const kick = () => { if (!raf && !dead) { last = performance.now(); raf = requestAnimationFrame(loop); } };

    // The atlas takes the page's Plex Mono if it arrives within a moment, the fallback mono otherwise, and is redrawn
    // when Plex Mono lands. The headline never waits for either.
    await new Promise((done) => { setTimeout(done, 400); document.fonts.load(FONT).then(done, done); });
    if (dead) return;
    await new Promise<void>((done) => {
      const ro = new ResizeObserver(([e]) => {
        if (dead || gl.isContextLost()) return;
        // Resizing clears the canvas after this frame's draw, so draw again even while the loop runs.
        size(e);
        if (ok) render();
        done();
      });
      try { ro.observe(canvas, { box: "device-pixel-content-box" }); } catch { ro.observe(canvas); }
      undo.push(() => ro.disconnect());
    });
    if (dead) return;
    if (gl.isContextLost()) return fail();
    if (!document.fonts.check(FONT)) document.fonts.load(FONT).then(() => { if (ok) { drawAtlas(); if (!raf) render(); } });
    ok = true;
    render();

    // A slow main thread can batch entries; the last one is the current state.
    const io = new IntersectionObserver((entries) => { seen = entries[entries.length - 1].isIntersecting; kick(); });
    io.observe(stage);
    // Reduced motion switched on mid-visit stops the loop at REST; switched off, the loop goes on from there.
    const onMotion = () => {
      reduce = motion.matches;
      if (!ok) return;
      if (reduce) { t = REST; render(); } else kick();
    };
    motion.addEventListener("change", onMotion);
    undo.push(() => motion.removeEventListener("change", onMotion));
    // The copy can reflow without the canvas resizing (a font arriving), and the dot follows the headline.
    const copy = new ResizeObserver(() => { if (ok) { layout(); if (!raf) render(); } });
    for (const { el } of keepEls) copy.observe(el);
    document.addEventListener("visibilitychange", kick);
    undo.push(() => { io.disconnect(); copy.disconnect(); document.removeEventListener("visibilitychange", kick); });
    if (matchMedia("(pointer: fine)").matches) {
      const aim = (e: PointerEvent) => { aimX = (e.clientX / innerWidth) * 2 - 1; aimY = (e.clientY / innerHeight) * 2 - 1; pointerOn = true; };
      const away = () => { pointerOn = false; };
      addEventListener("pointermove", aim, { passive: true });
      document.documentElement.addEventListener("pointerleave", away);
      undo.push(() => { removeEventListener("pointermove", aim); document.documentElement.removeEventListener("pointerleave", away); });
    }
  };

  // The mark starts after the first paint, so the headline never waits on the GPU.
  let timer = 0;
  const first = requestAnimationFrame(() => { timer = window.setTimeout(() => void run(), 0); });
  undo.push(() => { cancelAnimationFrame(first); clearTimeout(timer); });
  return () => {
    dead = true;
    cancelAnimationFrame(raf);
    for (const f of undo.splice(0).reverse()) f();
  };
}
