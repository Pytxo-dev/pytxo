import type { HitlDto } from "./types";

export type ApprovalPresentation = {
  title: string;
  category: string;
  approveLabel: string;
  denyLabel: string;
  consequence: string;
  approvedMessage: string;
  deniedMessage: string;
};

type PendingApprovalPresentation = Omit<
  ApprovalPresentation,
  "approvedMessage" | "deniedMessage"
>;

const ACTION_PRESENTATIONS: Record<string, PendingApprovalPresentation> = {
  "fs.write_outside_root": {
    title: "Allow write outside workspace",
    category: "Filesystem",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This write targets a path outside the repository boundary.",
  },
  "fs.delete": {
    title: "Allow recursive delete",
    category: "Filesystem",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This command was classified as a recursive filesystem delete.",
  },
  "fs.permission": {
    title: "Allow permission change",
    category: "Filesystem",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This command changes filesystem ownership or permissions.",
  },
  "git.destructive": {
    title: "Allow destructive Git action",
    category: "Git",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This command can rewrite or discard Git history.",
  },
  "git.push": {
    title: "Allow Git push",
    category: "Git",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This action writes local commits to a remote Git repository.",
  },
  "net.egress": {
    title: "Allow network access",
    category: "Network",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This command was classified as an outbound network request.",
  },
  "net.bind": {
    title: "Allow network tool",
    category: "Network",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This command invokes a network listener or connection tool.",
  },
  "proc.infrastructure": {
    title: "Allow infrastructure mutation",
    category: "Process",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This command can modify deployed infrastructure.",
  },
  "proc.docker": {
    title: "Allow container command",
    category: "Process",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This command invokes a container runtime.",
  },
  "proc.package_install": {
    title: "Allow package installation",
    category: "Process",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This command installs software into the current environment.",
  },
  "mcp.tool": {
    title: "Allow MCP tool call",
    category: "MCP",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence: "This request invokes an MCP tool or reads an MCP resource.",
  },
};

/**
 * Presentation contract for the action codes emitted by `pytxo-runner`'s
 * Galaxy HITL gate. Unknown codes remain visible and receive neutral wording
 * so Desktop cannot silently mislabel a new policy action as a sandbox flush.
 */
export function approvalPresentation(approval: HitlDto): ApprovalPresentation {
  const action = approval.action.trim().toLowerCase();
  if (action === "blast.flush" || action.includes("flush blast shield")) {
    return {
      title: "Flush Blast Shield workspace",
      category: "Sandbox",
      approveLabel: "Approve & flush",
      denyLabel: "Deny & discard",
      consequence:
        "Approving writes the isolated workspace changes into the repository. Denying discards them.",
      approvedMessage: "Sandbox flush can proceed.",
      deniedMessage: "Isolated changes will not be flushed.",
    };
  }

  const matched = ACTION_PRESENTATIONS[action] ?? {
    title: approval.action,
    category: "Approval",
    approveLabel: "Approve action",
    denyLabel: "Deny action",
    consequence:
      "This action is blocked by the current permission profile until you decide.",
  };
  return {
    ...matched,
    approvedMessage: "The blocked action can proceed.",
    deniedMessage: "The blocked action will not run.",
  };
}
