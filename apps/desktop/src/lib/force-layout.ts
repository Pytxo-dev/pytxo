import type { StructuralGraphDto } from "./types";

type Vec3 = { x: number; y: number; z: number };

/** Simple 3D force-directed layout (spring + repulsion). */
export function forceLayout3d(
  nodes: StructuralGraphDto["nodes"],
  edges: StructuralGraphDto["edges"],
  iterations = 60,
): Map<string, { x: number; y: number; z: number }> {
  const positions = new Map<string, Vec3>();
  const n = nodes.length;
  if (n === 0) return new Map();

  nodes.forEach((node, i) => {
    const angle = (i / n) * Math.PI * 2;
    positions.set(node.id, {
      x: Math.cos(angle) * 3,
      y: Math.sin(angle) * 3,
      z: (Math.random() - 0.5) * 2,
    });
  });

  const nodeIds = new Set(nodes.map((n) => n.id));

  for (let iter = 0; iter < iterations; iter++) {
    const forces = new Map<string, Vec3>();
    for (const id of nodeIds) {
      forces.set(id, { x: 0, y: 0, z: 0 });
    }

    // Repulsion
    const ids = [...nodeIds];
    for (let i = 0; i < ids.length; i++) {
      for (let j = i + 1; j < ids.length; j++) {
        const a = positions.get(ids[i])!;
        const b = positions.get(ids[j])!;
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;
        let dist = Math.sqrt(dx * dx + dy * dy + dz * dz) || 0.01;
        const repulse = 0.8 / (dist * dist);
        dx = (dx / dist) * repulse;
        dy = (dy / dist) * repulse;
        dz = (dz / dist) * repulse;
        const fa = forces.get(ids[i])!;
        const fb = forces.get(ids[j])!;
        fa.x += dx;
        fa.y += dy;
        fa.z += dz;
        fb.x -= dx;
        fb.y -= dy;
        fb.z -= dz;
      }
    }

    // Springs along edges
    for (const edge of edges) {
      if (!nodeIds.has(edge.from) || !nodeIds.has(edge.to)) continue;
      const a = positions.get(edge.from)!;
      const b = positions.get(edge.to)!;
      let dx = b.x - a.x;
      let dy = b.y - a.y;
      let dz = b.z - a.z;
      let dist = Math.sqrt(dx * dx + dy * dy + dz * dz) || 0.01;
      const spring = (dist - 1.2) * 0.08;
      dx = (dx / dist) * spring;
      dy = (dy / dist) * spring;
      dz = (dz / dist) * spring;
      const fa = forces.get(edge.from)!;
      const fb = forces.get(edge.to)!;
      fa.x += dx;
      fa.y += dy;
      fa.z += dz;
      fb.x -= dx;
      fb.y -= dy;
      fb.z -= dz;
    }

    // Integrate
    const damp = 0.85 - iter / iterations * 0.2;
    for (const id of nodeIds) {
      const p = positions.get(id)!;
      const f = forces.get(id)!;
      p.x += f.x * damp;
      p.y += f.y * damp;
      p.z += f.z * damp;
      const r = Math.sqrt(p.x * p.x + p.y * p.y + p.z * p.z);
      if (r > 6) {
        p.x *= 6 / r;
        p.y *= 6 / r;
        p.z *= 6 / r;
      }
    }
  }

  const out = new Map<string, { x: number; y: number; z: number }>();
  for (const [id, p] of positions) {
    out.set(id, p);
  }
  return out;
}
