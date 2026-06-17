import fs from "node:fs";
const file = process.argv[2];
if (!file) process.exit(1);
const text = fs.readFileSync(file, "utf8");
for (const line of text.split(/\n/)) {
  const m = line.match(/^([A-Z0-9_]+)=(.*)$/);
  if (!m) continue;
  const key = m[1];
  let val = m[2];
  if (val.startsWith('"') && val.endsWith('"')) val = val.slice(1, -1);
  if (!key.includes("CLERK") && !key.includes("MBCZ") && !key.includes("PADDLE") && !key.includes("LINK")) continue;
  const preview = val ? `${val.slice(0, 12)}… (${val.length} chars)` : "EMPTY";
  console.log(`${key}: ${preview}`);
}
