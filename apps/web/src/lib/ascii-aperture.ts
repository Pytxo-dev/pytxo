// Character shell and diagonal opening shared with Desktop's ApertureGlyph.
// This is decorative brand motion, never an execution or verification signal.
export const APERTURE_COLORS = ["#7ee1ed", "#b4b9ff", "#bd9eef", "#e9bb91"];
export const APERTURE_POINTS = Array.from({ length: 520 }, (_, n) => {
  const y = 1 - 2 * (n + 0.5) / 520;
  const radius = Math.sqrt(1 - y * y);
  const angle = n * Math.PI * (3 - Math.sqrt(5));
  return { x: Math.cos(angle) * radius, y, z: Math.sin(angle) * radius, char: ".:+*=x"[n % 6] };
});

export function apertureFrame(phase: number) {
  return APERTURE_POINTS.map(p => {
    const x = p.x * Math.cos(phase) + p.z * Math.sin(phase);
    const z = p.z * Math.cos(phase) - p.x * Math.sin(phase);
    return { ...p, x, z, color: p.y < -0.35 ? 0 : p.y < 0.25 ? 1 : x > 0.45 ? 3 : 2 };
  }).filter(p => p.z >= -0.15 && Math.abs(p.y - 0.46 * p.x) >= 0.18);
}
