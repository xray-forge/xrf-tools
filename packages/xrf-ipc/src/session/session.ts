import { isTauri } from "@tauri-apps/api/core";
import { Logger } from "@xrf/logging";
import { Maybe } from "@xrf/types";

import { SessionScopeService } from "#/session/session-scope.service";
import { IScopedSession, ISessionIdentity } from "#/session/session.types";
import { SessionId } from "#/types/xrf-app-core";

/**
 * Owns native openings through completion and teardown, including cancelled frontend flows. Each opening is bound to
 * the scope of the tool that owns it before it opens, so the backend closes it with the tool or the window, and the
 * session is retired with that scope: a service closes its openings itself only when it lets them go while its tool
 * stays.
 */
export class Session implements IScopedSession {
  private readonly owned: Set<SessionId> = new Set();
  private readonly pending: Set<SessionId> = new Set();

  private revision: number = 0;

  /**
   * @param closeSessions - Backend command releasing the named openings, typically a generated `close` command.
   * @param scopeService - The scope of the tool owning the openings.
   */
  public constructor(
    private readonly closeSessions: (sessionIds: Array<SessionId>) => Promise<unknown>,
    private readonly scopeService: SessionScopeService
  ) {
    scopeService.register(this);
  }

  /** The tool the openings are held for, which a restore names to attach to what that tool holds. */
  public get tool(): string {
    return this.scopeService.tool;
  }

  /**
   * Takes responsibility for an opening this view did not allocate.
   *
   * Restoration reads the committed opening straight from the backend, and a settled job reports one that outlived a
   * reload; neither passes through `open`, so neither is owned until it is adopted.
   *
   * @param session - Identity to take over, or nothing when the backend holds none.
   */
  public adopt(session: Maybe<ISessionIdentity>): void {
    if (session) {
      this.owned.add(session.sessionId);
    }
  }

  /**
   * Allocates an identity before dispatch and releases superseded results.
   *
   * @param command - Command taking the new identity first, or an adapter for a request object.
   * @param args - Remaining command arguments.
   * @returns The command result; the caller's flow controls whether it reaches the view.
   */
  public async open<T extends ISessionIdentity, Args extends Array<unknown>>(
    command: (sessionId: SessionId, ...args: Args) => Promise<T>,
    ...args: Args
  ): Promise<T> {
    const sessionId: SessionId = crypto.randomUUID();
    const revision: number = ++this.revision;

    let response: T;

    this.owned.add(sessionId);
    this.pending.add(sessionId);

    try {
      await this.scopeService.bind(sessionId);
      response = await command(sessionId, ...args);
    } catch (error) {
      // Only the open is guarded here, so reaching this means the backend committed nothing there is to release.
      this.owned.delete(sessionId);

      throw error;
    } finally {
      this.pending.delete(sessionId);
    }

    if (revision === this.revision) {
      // The backend holds every opening apart, so the ones this publication replaces are closed rather than forgotten;
      // unfinished calls retain their own identities, and one that fails to close is retried by the next close.
      const replaced: Array<SessionId> = [...this.owned].filter((id) => id !== sessionId && !this.pending.has(id));

      if (replaced.length > 0) {
        this.closeOwned(replaced).catch((error: unknown) => {
          Logger.error("Failed to release replaced sessions:", error);
        });
      }

      return response;
    }

    // A close finished before this publication reached us, so releasing it falls to this call; it stays owned, and so
    // retryable, only where the release fails.
    await this.closeOwned([sessionId]);

    return response;
  }

  /**
   * Releases every owned opening, allocated or adopted. Failed releases remain retryable.
   *
   * @returns Resolves once the backend has dropped them.
   */
  public async close(): Promise<void> {
    this.revision += 1;

    const ids: Array<SessionId> = [...this.owned];

    if (ids.length === 0) {
      return;
    }

    await this.closeOwned(ids);
  }

  /**
   * Releases openings, owned no longer while the release is under way so another close does not send them again, and
   * owned again where it fails, so the next close retries them.
   */
  private async closeOwned(ids: Array<SessionId>): Promise<void> {
    for (const id of ids) {
      this.owned.delete(id);
    }

    try {
      await this.closeSessions(ids);
    } catch (error) {
      for (const id of ids) {
        this.owned.add(id);
      }

      throw error;
    }
  }

  /**
   * Forgets every opening as the scope's release closes them on the backend, so nothing is sent for each; a publication
   * landing later closes itself, as after any close.
   */
  public retire(): void {
    this.revision += 1;
    this.owned.clear();
  }

  /**
   * Starts the release that a teardown cannot wait for.
   *
   * Deactivation is synchronous, so a failure is reported here rather than propagated: there is no longer a surface to
   * report it to, and anything still owned is released again by the next close.
   */
  public release(): void {
    if (!isTauri()) {
      return;
    }

    this.close().catch((error: unknown) => {
      Logger.error("Failed to release a session on deactivation:", error);
    });
  }
}
