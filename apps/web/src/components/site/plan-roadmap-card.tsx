import { CheckIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

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
    <Card className="h-full border-white/10 bg-card/30">
      <CardHeader>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <CardTitle className="text-lg">{name}</CardTitle>
          <Badge variant={statusVariant}>{status}</Badge>
        </div>
        <CardDescription className="text-left leading-relaxed">{detail}</CardDescription>
      </CardHeader>
      <CardContent>
        <ul className="flex flex-col gap-2 text-sm text-muted-foreground">
          {highlights.map((highlight) => (
            <li key={highlight} className="flex gap-2">
              <CheckIcon aria-hidden="true" className="size-4 shrink-0 text-primary" />
              {highlight}
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}
