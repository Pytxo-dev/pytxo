import type { UpdateState } from "./update-controller";

export function isUpdateBusy(state: UpdateState): boolean {
  return ["checking", "preflight", "downloading", "installing", "restarting"].includes(state.phase);
}

export function isUpdateCheckDisabled(state: UpdateState): boolean {
  return isUpdateBusy(state)
    || state.phase === "ready-to-install"
    || state.phase === "restart-required"
    || state.phase === "unavailable";
}

export function shouldShowUpdateCheck(state: UpdateState, compact: boolean): boolean {
  if (state.version) return false;
  return !compact || Boolean(state.error);
}

export function updateActionLabel(state: UpdateState): string {
  if (state.phase === "restart-required") return "Restart Pytxo";
  if (state.phase === "ready-to-install") return "Install and restart";
  return "Download update";
}

export function updatePhaseMessage(state: UpdateState): string {
  if (state.phase === "unavailable") return "Updates are available in the installed Desktop app.";
  if (state.phase === "checking") return "Checking for updates…";
  if (state.phase === "preflight") return "Checking workspace activity before updating…";
  if (state.phase === "current") return "No newer version is available on this update channel.";
  if (state.phase === "downloading") {
    const percent = state.total && state.total > 0
      ? Math.min(100, Math.floor(state.downloaded / state.total * 100))
      : null;
    return `Downloading v${state.version}${percent !== null ? ` · ${percent}%` : "…"}`;
  }
  if (state.phase === "ready-to-install") {
    return `v${state.version} is downloaded and verified. Installing closes Pytxo, then Windows asks for administrator permission.`;
  }
  if (state.phase === "installing") return "Opening the Windows installer… Accept the administrator prompt to finish.";
  if (state.phase === "restarting") return "Restarting Pytxo…";
  if (state.phase === "restart-required") return "Restart Pytxo to check the installed version.";
  if (state.version) return `v${state.version} is available. Download it while Pytxo stays open.`;
  return "Check for a newer Desktop build.";
}
