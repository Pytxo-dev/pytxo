/**
 * Keep Windows' extended-length prefix inside the native path layer.
 *
 * Rust canonicalization intentionally returns `\\?\C:\…` (or
 * `\\?\UNC\server\share`) so filesystem calls remain reliable. That prefix is
 * implementation detail, though, and should never leak into reader-facing UI.
 */
export function displayPath(path: string): string {
  if (path.startsWith("\\\\?\\UNC\\")) return `\\\\${path.slice(8)}`;
  if (path.startsWith("\\\\?\\")) return path.slice(4);
  return path;
}
