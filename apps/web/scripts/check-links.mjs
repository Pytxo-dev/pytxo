import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const WEB_ROOT = fileURLToPath(new URL("../", import.meta.url));
const APP_ROOT = path.join(WEB_ROOT, "src", "app");
const CONTENT_ROOT = path.join(WEB_ROOT, "content", "docs");
const PUBLIC_ROOT = path.join(WEB_ROOT, "public");

async function walk(directory, accepted) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await walk(target, accepted)));
    } else if (accepted(target)) {
      files.push(target);
    }
  }
  return files;
}

function normalizeRoute(raw) {
  const route = raw.split("#", 1)[0].split("?", 1)[0];
  if (!route || route === "/") return "/";
  return route.replace(/\/+$/, "");
}

function docsRoute(file) {
  const relative = path.relative(CONTENT_ROOT, file).replaceAll("\\", "/");
  const withoutExtension = relative.replace(/\.mdx$/, "");
  const suffix = withoutExtension === "index"
    ? ""
    : withoutExtension.endsWith("/index")
      ? withoutExtension.slice(0, -"/index".length)
      : withoutExtension;
  return normalizeRoute(`/docs/${suffix}`);
}

function appRoute(file) {
  const parts = path
    .relative(APP_ROOT, path.dirname(file))
    .split(path.sep)
    .filter((part) => part && !(part.startsWith("(") && part.endsWith(")")));
  const staticParts = parts.filter((part) => !part.startsWith("["));
  return normalizeRoute(`/${staticParts.join("/")}`);
}

async function publicAssetExists(route) {
  const relative = decodeURIComponent(route).replace(/^\/+/, "");
  const target = path.resolve(PUBLIC_ROOT, relative);
  if (!target.startsWith(path.resolve(PUBLIC_ROOT) + path.sep)) return false;
  try {
    return (await stat(target)).isFile();
  } catch (error) {
    if (error?.code === "ENOENT") return false;
    throw error;
  }
}

const docsFiles = await walk(CONTENT_ROOT, (file) => file.endsWith(".mdx"));
const appPages = await walk(APP_ROOT, (file) => path.basename(file) === "page.tsx");
const sourceFiles = [
  ...docsFiles,
  ...(await walk(path.join(WEB_ROOT, "src"), (file) => /\.(tsx?|mdx)$/.test(file))),
];

const routes = new Set([
  ...docsFiles.map(docsRoute),
  ...appPages.map(appRoute),
]);
const markdownLink = /\]\((\/[^)\s]+)(?:\s+"[^"]*")?\)/g;
const jsxLink = /\b(?:href|src)=["'](\/[^"']+)["']/g;
const failures = [];
let checked = 0;

for (const file of sourceFiles) {
  const content = await readFile(file, "utf8");
  const references = [
    ...[...content.matchAll(markdownLink)].map((match) => match[1]),
    ...[...content.matchAll(jsxLink)].map((match) => match[1]),
  ];

  for (const reference of references) {
    const route = normalizeRoute(reference);
    if (
      route.startsWith("/api/") ||
      route.startsWith("/_next/") ||
      route.includes("${")
    ) {
      continue;
    }

    checked += 1;
    if (routes.has(route) || (await publicAssetExists(route))) continue;
    failures.push(`${path.relative(WEB_ROOT, file)} -> ${reference}`);
  }
}

if (failures.length > 0) {
  throw new Error(`Broken internal links:\n${failures.join("\n")}`);
}

console.log(`Checked ${checked} internal links across ${sourceFiles.length} web source files.`);
