import Image from "next/image";
import Link from "next/link";
import { ArrowUpRight, Check, KeyRound, ShieldCheck } from "lucide-react";

const READY_STATES = [
  {
    title: "Agent sessions stay vendor-owned",
    detail: "ChatGPT connects through Codex. Claude, Cursor, Gemini, and OpenCode keep their own sessions.",
    icon: ShieldCheck,
  },
  {
    title: "API billing stays separate",
    detail: "DeepSeek and other direct providers use an explicit environment-variable reference for the selected run.",
    icon: KeyRound,
  },
  {
    title: "Readiness checks are non-billable",
    detail: "Pytxo checks installed CLIs and redacted session state without sending a model request.",
    icon: Check,
  },
] as const;

export function AgentReadiness() {
  return (
    <section className="section-pad mx-auto max-w-7xl" aria-labelledby="agent-readiness-title">
      <div className="grid gap-9 lg:grid-cols-[minmax(0,0.72fr)_minmax(0,1.28fr)] lg:items-center lg:gap-14">
        <div className="max-w-xl">
          <p className="text-sm font-medium tracking-wide text-muted-foreground">
            Real agent readiness
          </p>
          <h2 id="agent-readiness-title" className="mt-3 text-3xl sm:text-4xl">
            Keep the accounts you already trust
          </h2>
          <p className="mt-4 leading-relaxed text-muted-foreground">
            Pytxo coordinates agent processes without becoming another credential vault. Connect
            through each official CLI, recheck its redacted status, then choose a ready agent in
            a mission.
          </p>

          <ul className="mt-8 grid gap-5">
            {READY_STATES.map((item) => {
              const Icon = item.icon;
              return (
                <li key={item.title} className="grid grid-cols-[1.25rem_1fr] gap-3">
                  <span className="mt-1 text-muted-foreground">
                    <Icon className="size-4" aria-hidden />
                  </span>
                  <span>
                    <strong className="block text-sm font-semibold">{item.title}</strong>
                    <span className="mt-1 block text-sm leading-relaxed text-muted-foreground">
                      {item.detail}
                    </span>
                  </span>
                </li>
              );
            })}
          </ul>

          <Link
            href="/docs/getting-started/desktop-setup"
            className="mt-8 inline-flex items-center gap-1.5 text-sm font-medium text-primary transition-colors hover:text-foreground"
          >
            Set up Pytxo Desktop <ArrowUpRight className="size-3.5" aria-hidden />
          </Link>
        </div>

        <div
          className="overflow-hidden rounded-[4px] border border-border bg-card"
          data-testid="agent-readiness-product"
        >
          <Image
            src="/product/integrations-1600x1000.png"
            alt="Pytxo Desktop Agents showing installed coding agents, vendor-owned sessions, and readiness actions"
            width={1600}
            height={1000}
            className="h-auto w-full"
            sizes="(max-width: 1023px) 100vw, 58vw"
          />
        </div>
      </div>
    </section>
  );
}
