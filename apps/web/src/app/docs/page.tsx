import type { Metadata } from "next";

import { docsMetadata, renderDocsPage } from "@/lib/docs-page";
import { assertDocsSource } from "@/lib/docs";

export const dynamic = "force-static";

export function generateMetadata(): Metadata {
  assertDocsSource();
  return docsMetadata([]);
}

export default async function DocsIndexPage() {
  assertDocsSource();
  return renderDocsPage([]);
}
