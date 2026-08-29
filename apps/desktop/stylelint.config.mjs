/**
 * Guards the epistemic state contract at the CSS layer
 * (docs/05-adr/ADR-0038-epistemic-state-contract.md).
 *
 * The regression this exists to prevent actually shipped: every agent lane in
 * OperationsLanes.svelte rendered `width: 72%`, an identical progress fill in
 * every state, for a quantity the orchestrator never computes. A literal
 * percentage in a state or progress visual is a fabricated quantity by
 * definition, because a real one has to arrive from the DTO as an inline style
 * or a custom property.
 */
const STATE_VISUAL_SELECTORS = [
  /\.(timeline|progress|fill|bar|track|meter|gauge|lane)/,
  /\[data-state/,
];

const PERCENTAGE_WIDTH = /^-?\d+(\.\d+)?%$/;

const ruleName = "pytxo/no-literal-percentage-in-state-visual";

const plugin = {
  ruleName,
  rule: () => (root, result) => {
    root.walkDecls(/^(width|height|inline-size|block-size)$/, (decl) => {
      if (!PERCENTAGE_WIDTH.test(decl.value.trim())) return;
      if (decl.value.trim() === "100%") return;

      const selector = decl.parent?.selector ?? "";
      if (!STATE_VISUAL_SELECTORS.some((pattern) => pattern.test(selector))) return;

      result.warn(
        `Literal "${decl.prop}: ${decl.value}" in state visual "${selector}". A progress or ` +
          "state dimension must come from a bound numeric source (inline style or custom " +
          "property), never a hardcoded percentage. See ADR-0038.",
        { node: decl, result, ruleName, severity: "error" },
      );
    });
  },
};

plugin.rule.ruleName = ruleName;
plugin.rule.messages = {};
plugin.rule.meta = { url: "docs/05-adr/ADR-0038-epistemic-state-contract.md" };

/** @type {import('stylelint').Config} */
export default {
  customSyntax: "postcss-html",
  plugins: [plugin],
  rules: {
    [ruleName]: true,
  },
};
