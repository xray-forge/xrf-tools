// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { SessionId, SessionScope } from "@/core/ipc/types/xrf-app";

/** Commands */
export const sessionsCommands = {
  /**
   * Bind an opening to the tool that asks for it, before its open, so whatever releases the tool's scope closes it; an
   * opening not bound is refused.
   */
  bind: (sessionId: SessionId, scope: SessionScope) =>
    __TAURI_INVOKE<void>("plugin:sessions|bind", { sessionId, scope }),
  /**
   * Close every opening a tool held, in every domain, as the tool is left; answers how many there were.
   *
   * Synchronous, so the tool's bindings go in the order the page sent its calls: an opening bound after it, by the tool
   * entered again, is not one it closes. What the openings held is dropped on the pool.
   */
  releaseScope: (scope: SessionScope) => __TAURI_INVOKE<number>("plugin:sessions|release_scope", { scope }),
  /**
   * Keep only the openings of a window's live tools, closing what any other tool of it left, as a page starts and knows
   * which tools it runs: a page that reloaded could not release what it was leaving. Answers how many were closed.
   *
   * Synchronous, as `release_scope` is, so it unbinds in the order the page sent its calls.
   */
  retainScopes: (window: string, live: Array<SessionScope>) =>
    __TAURI_INVOKE<number>("plugin:sessions|retain_scopes", { window, live }),
};
