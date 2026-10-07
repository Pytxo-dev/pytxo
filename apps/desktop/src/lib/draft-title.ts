/** Saved-request title: whole words up to `max` characters, marked when cut. */
export function draftTitle(text: string, max = 72): string {
  const flat = text.trim().replace(/\s+/g, " ");
  if (flat.length <= max) return flat;
  const cut = flat.slice(0, max - 1);
  const space = cut.lastIndexOf(" ");
  return `${(space > max / 2 ? cut.slice(0, space) : cut).trimEnd()}…`;
}
