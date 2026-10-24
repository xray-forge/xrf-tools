// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { IIpcCommandSignature } from "@/core/ipc/signature";
import { SessionId } from "@/core/ipc/types/xrf-app-core";

/** The wire name of every command of the `sessions` plugin. */
export enum ESessionsCommand {
  BIND = "plugin:sessions|bind",
  RELEASE_SCOPE = "plugin:sessions|release_scope",
  RETAIN_SCOPES = "plugin:sessions|retain_scopes",
}

/** Commands */
export const sessionsCommands = {
  /**
   * Bind an opening to the tool that asks for it, in the window it asks from, before its open, so whatever releases the
   * tool's scope closes it; an opening not bound is refused.
   */
  bind: (sessionId: SessionId, tool: string) => __TAURI_INVOKE<void>(ESessionsCommand.BIND, { sessionId, tool }),
  /**
   * Close every opening a tool of the asking window held, in every domain, as the tool is left; answers how many there
   * were.
   *
   * Synchronous, so the tool's bindings go in the order the page sent its calls: an opening bound after it, by the tool
   * entered again, is not one it closes. What the openings held is dropped on the pool.
   */
  releaseScope: (tool: string) => __TAURI_INVOKE<number>(ESessionsCommand.RELEASE_SCOPE, { tool }),
  /**
   * Keep only the openings of the asking window's live tools, closing what any other tool of it left, as a page starts
   * and knows which tools it runs: a page that reloaded could not release what it was leaving. Answers how many were
   * closed.
   *
   * Synchronous, as `release_scope` is, so it unbinds in the order the page sent its calls.
   */
  retainScopes: (tools: Array<string>) => __TAURI_INVOKE<number>(ESessionsCommand.RETAIN_SCOPES, { tools }),
};

/** What each command of the `sessions` plugin takes and resolves to, keyed by its wire name. */
export interface ISessionsCommandSignatures {
  [ESessionsCommand.BIND]: IIpcCommandSignature<typeof sessionsCommands.bind, ["sessionId", "tool"]>;
  [ESessionsCommand.RELEASE_SCOPE]: IIpcCommandSignature<typeof sessionsCommands.releaseScope, ["tool"]>;
  [ESessionsCommand.RETAIN_SCOPES]: IIpcCommandSignature<typeof sessionsCommands.retainScopes, ["tools"]>;
}
