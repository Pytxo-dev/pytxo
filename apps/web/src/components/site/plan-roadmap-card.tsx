"use client";

import { SparklesIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card";

type PlanRoadmapCardProps = {
  name: string;
  status: string;
  statusVariant: "default" | "secondary";
  highlights: readonly string[];
  detail: string;
};

export function PlanRoadmapCard({
  name,
  status,
  statusVariant,
  highlights,
  detail,
}: PlanRoadmapCardProps) {
  return (
    <HoverCard openDelay={120} closeDelay={80}>
      <HoverCardTrigger asChild>
        <div className="block h-full rounded-xl outline-none">
          <Card className="glass-panel chroma-edge-top h-full border-white/8 transition duration-300 hover:chroma-glow">
            <CardHeader>
              <div className="flex items-center justify-between gap-2">
                <CardTitle className="text-lg">{name}</CardTitle>
                <Badge variant={statusVariant}>{status}</Badge>
              </div>
              <CardDescription className="text-left">{highlights[0]}</CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="flex flex-col gap-2 text-sm text-muted-foreground">
                {highlights.map((h) => (
                  <li key={h} className="flex gap-2">
                    <SparklesIcon className="size-4 shrink-0 text-primary" />
                    {h}
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>
        </div>
      </HoverCardTrigger>
      <HoverCardContent className="glass-panel w-80 border-white/10">
        <p className="text-sm text-muted-foreground">{detail}</p>
      </HoverCardContent>
    </HoverCard>
  );
}
