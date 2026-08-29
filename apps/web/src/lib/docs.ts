import { source } from "@/lib/source";

/** Fail the web build if Fumadocs did not load the curated MDX tree. */
export const MIN_DOCS_PAGES = 20;

type DocsPageRecord = {
  slugs?: string[];
  url: string;
};

export function pageSlugParts(page: DocsPageRecord): string[] {
  if (Array.isArray(page.slugs)) return page.slugs;
  const stripped = page.url.replace(/^\/docs\/?/, "");
  return stripped ? stripped.split("/").filter(Boolean) : [];
}

export function assertDocsSource() {
  const pages = source.getPages();
  if (pages.length < MIN_DOCS_PAGES) {
    throw new Error(
      `Fumadocs loaded ${pages.length} pages (need >= ${MIN_DOCS_PAGES}). Check content/docs and fumadocs-mdx generation.`,
    );
  }
  return pages;
}

export function docsNestedStaticParams() {
  const pages = assertDocsSource();
  const generated = source.generateParams();
  const nested = generated
    .map((entry) => {
      const slug = (entry as { slug?: unknown }).slug;
      return Array.isArray(slug) ? slug : null;
    })
    .filter((slug): slug is string[] => slug !== null && slug.length > 0)
    .map((slug) => ({ slug }));

  const fromPages = pages
    .map((page) => pageSlugParts(page))
    .filter((slug) => slug.length > 0)
    .map((slug) => ({ slug }));

  const params = nested.length >= fromPages.length ? nested : fromPages;
  if (params.length < MIN_DOCS_PAGES - 1) {
    throw new Error(
      `Fumadocs generateStaticParams produced ${params.length} nested docs routes (need >= ${MIN_DOCS_PAGES - 1}).`,
    );
  }
  console.info(`[docs] static nested params: ${params.length} (source pages: ${pages.length})`);
  return params;
}
