import { invoke } from "@tauri-apps/api/core";
import { normalizeIpcError } from "./ipc";
export type TerminalInfo = { id: string; domain_id: string; cwd: string; owner: string; state: "running" | "ending" | "ended"; exit_code: number | null };
export type TerminalPage = { session: TerminalInfo; start: number; next: number; gap: boolean; data_base64: string; error: string | null };
export const hasNativeTerminal = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!hasNativeTerminal()) throw new Error("Interactive workspace terminals require native Pytxo Desktop. Browser previews cannot create a shell.");
  try { return await invoke<T>(command, args); } catch (e) { throw normalizeIpcError(e); }
}
export const workspaceTerminal = {
  create: (domainId: string) => call<TerminalInfo>("workspace_terminal_create", { domainId }),
  list: () => hasNativeTerminal() ? call<TerminalInfo[]>("workspace_terminal_list") : Promise.resolve([]),
  read: (sessionId: string, domainId: string, after: number) => call<TerminalPage>("workspace_terminal_read", { sessionId, domainId, after }),
  input: (sessionId: string, domainId: string, data: string) => call<void>("workspace_terminal_input", { sessionId, domainId, data }),
  resize: (sessionId: string, domainId: string, rows: number, cols: number) => call<void>("workspace_terminal_resize", { sessionId, domainId, rows, cols }),
  end: (sessionId: string, domainId: string) => call<void>("workspace_terminal_end", { sessionId, domainId }),
};
