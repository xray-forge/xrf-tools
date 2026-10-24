import { isTauri } from "@tauri-apps/api/core";
import { inject, Injectable, InjectionToken, OnDeactivation } from "@wirestate/core";

import { sessionsCommands } from "@/core/ipc/commands/sessions";
import { SessionId, SessionScope } from "@/core/ipc/types/xrf-app";
import { Logger } from "@/lib/logging";
import { resolveAppWindow } from "@/lib/tauri/app-window";

/** The tool a container's services hold their openings for: an application's id, or the shell's. */
export const SESSION_TOOL: InjectionToken<string> = new InjectionToken<string>("SessionTool");

/** What the shell itself holds open, beside every application. */
export const SHELL_SESSION_TOOL: string = "shell";

/**
 * The scope a container's openings are held in: its window and tool, named alike across the window's page loads, so a
 * reloaded page attaches to what its tool held. Released as the container goes, which is when its tool is left; a page
 * that reloads releases nothing, since the backend holds what it opened.
 */
@Injectable()
export class SessionScopeService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly scope: SessionScope;

  /**
   * @param tool - The tool the scope is held for.
   */
  public constructor(tool: string = inject(SESSION_TOOL)) {
    // Outside a Tauri webview there is no window, and no backend to hold a scope.
    this.scope = { tool, window: resolveAppWindow()?.label ?? "" };
  }

  /**
   * Binds an opening to this scope before it opens, so a scope released while it opens closes it.
   *
   * @param sessionId - The opening's identity.
   * @returns Resolves once the backend holds the binding.
   */
  public async bind(sessionId: SessionId): Promise<void> {
    if (isTauri()) {
      await sessionsCommands.bind(sessionId, this.scope);
    }
  }

  /** Closes every opening the scope holds, in every domain, as its tool is left. */
  @OnDeactivation()
  public release(): void {
    if (!isTauri()) {
      return;
    }

    sessionsCommands.releaseScope(this.scope).catch((error: unknown) => {
      this.log.error("Failed to release a tool's openings:", error);
    });
  }
}
