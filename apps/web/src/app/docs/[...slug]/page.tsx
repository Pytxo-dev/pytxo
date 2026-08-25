import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { docsNestedStaticParams } from "@/lib/docs";
import { docsMetadata, renderDocsPage } from "@/lib/docs-page";
import { source } from "@/lib/source";

export const dynamic = "force-static";
export const dynamicParams = false;

export function generateStaticParams() {
  return docsNestedStaticParams();
}

export async function generateMetadata(props: {
  params: Promise<{ slug: string[] }>;
}): Promise<Metadata> {
  const { slug } = await props.params;
  if (!source.getPage(slug)) notFound();
  return docsMetadata(slug);
}

export default async function DocsSlugPage(props: {
  params: Promise<{ slug: string[] }>;
}) {
  const { slug } = await props.params;
  return renderDocsPage(slug);
}
