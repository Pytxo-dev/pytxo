import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

import { ChassisMark } from "@/components/site/chassis-mark";
import { NAV_LINKS } from "@/lib/site";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <>
          <ChassisMark className="size-[22px] text-foreground" />
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
