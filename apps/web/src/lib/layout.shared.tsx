import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import Image from "next/image";

import { NAV_LINKS } from "@/lib/site";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <>
          <Image
            src="/logo.png"
            alt=""
            width={22}
            height={22}
            className="rounded-sm"
          />
          Pytxo
        </>
      ),
      url: "/docs",
    },
    links: NAV_LINKS.filter((l) => l.href !== "/docs").map((l) => ({
      text: l.label,
      url: l.href,
      active: "none" as const,
    })),
  };
}
