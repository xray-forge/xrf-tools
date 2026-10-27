import { isTauri } from "@tauri-apps/api/core";
import { inject, Injectable, InjectionToken, OnDeactivation } from "@wirestate/core";
import { Logger } from "@xrf/logging";

import { sessionsCommands } from "#/commands/sessions";
import { IScopedSession } from "#/session/session.types";
import { SessionId } from "#/types/xrf-app-core";

/** The tool a container's services hold their openings for: an application's id, or the shell's. */
export const SESSION_TOOL: InjectionToken<string> = new InjectionToken<string>("SessionTool");

/** What the shell itself holds open, beside every application. */
export const SHELL_SESSION_TOOL: string = "shell";

/**
 * The scope a container's openings are held in: its tool, named alike across the window's page loads, so a reloaded
 * page attaches to what its tool held; the backend reads the window from the call itself. Released as the container
 * goes, which is when its tool is left: one call closes what every service of the tool held, and each service's
 * sessions are retired with it. A page that reloads releases nothing, since the backend holds what it opened.
 */
@Injectable()
export class SessionScopeService {
  public readonly log: Logger = new Logger("session-scope.service");

  private readonly sessions: Set<IScopedSession> = new Set();

  /**
   * @param tool - The tool the scope is held for.
   */
  public constructor(public readonly tool: string = inject(SESSION_TOOL)) {}

  /**
   * Binds an opening to this scope before it opens, so a scope released while it opens closes it.
   *
   * @param sessionId - The opening's identity.
   * @returns Resolves once the backend holds the binding.
   */
  public async bind(sessionId: SessionId): Promise<void> {
    if (isTauri()) {
      await sessionsCommands.bind(sessionId, this.tool);
    }
  }

  /**
   * Takes what a service holds for its tool as what the scope's release retires; each registers itself as it is made.
   *
   * @param session - The session, or the follower, to retire with the scope.
   */
  public register(session: IScopedSession): void {
    this.sessions.add(session);
  }

  /** Closes every opening the scope holds, in every domain, as its tool is left, and retires every session in it. */
  @OnDeactivation()
  public release(): void {
    for (const session of this.sessions) {
      session.retire();
    }

    if (!isTauri()) {
      return;
    }

    sessionsCommands.releaseScope(this.tool).catch((error: unknown) => {
      this.log.error("Failed to release a tool's openings:", error);
    });
  }
}
