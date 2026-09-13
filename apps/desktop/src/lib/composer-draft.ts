/** Inputs survive navigation in this window; plans and dispatch authority do not. */
export type ComposerDraft = {
  mission: string;
  source: "text" | "voice";
  domainId: string;
  adeId: string;
};
