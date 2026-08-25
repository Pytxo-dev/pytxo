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
  const lamp = statusVariant === "default" ? "live" : "idle";

  return (
    <article className="bezel-frame h-full">
      <div className="bezel-frame__bar">
        <span className="status-lamp" data-state={lamp} aria-hidden />
        <span>{status}</span>
      </div>
      <div className="flex flex-col gap-3 p-5">
        <h3 data-slot="card-title" className="text-lg font-semibold tracking-tight">
          {name}
        </h3>
        <p className="text-sm leading-relaxed text-muted-foreground">{detail}</p>
        <ul className="flex flex-col gap-1.5 font-mono text-[13px] text-foreground">
          {highlights.map((highlight) => (
            <li key={highlight} className="grid grid-cols-[0.75rem_minmax(0,1fr)] gap-2">
              <span className="text-muted-foreground" aria-hidden>
                ·
              </span>
              <span>{highlight}</span>
            </li>
          ))}
        </ul>
      </div>
    </article>
  );
}
