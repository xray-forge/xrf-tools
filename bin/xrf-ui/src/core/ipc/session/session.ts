import { isTauri } from "@tauri-apps/api/core";

import { ISessionIdentity } from "@/core/ipc/session/session.types";
import { SessionId } from "@/core/ipc/types/xrf-app";
import { Logger } from "@/lib/logging";
import { Maybe } from "@/lib/types/general";

/**
 * Owns native openings through completion and teardown, including cancelled frontend flows.
 */
export class Session {
  private readonly owned: Set<SessionId> = new Set();
  private readonly pending: Set<SessionId> = new Set();

  private revision: number = 0;

  /**
   * @param closeSessions - Backend command releasing the named openings, typically a generated `close` command.
   */
  public constructor(private readonly closeSessions: (sessionIds: Array<SessionId>) => Promise<unknown>) {}

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
      response = await command(sessionId, ...args);
    } catch (error) {
      // Only the open is guarded here, so reaching this means the backend committed nothing there is to release.
      this.owned.delete(sessionId);

      throw error;
    } finally {
      this.pending.delete(sessionId);
    }

    if (revision === this.revision) {
      // Successful publication replaced the committed value; unfinished calls retain their own identities.
      for (const id of this.owned) {
        if (id !== sessionId && !this.pending.has(id)) {
          this.owned.delete(id);
        }
      }

      return response;
    }

    // A close finished before this publication reached us, so releasing it falls to this call. Re-owned first because
    // that close already forgot it, which is what leaves it retryable when the release below fails.
    this.owned.add(sessionId);

    await this.closeSessions([sessionId]);

    this.owned.delete(sessionId);

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

    await this.closeSessions(ids);

    for (const id of ids) {
      this.owned.delete(id);
    }
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
