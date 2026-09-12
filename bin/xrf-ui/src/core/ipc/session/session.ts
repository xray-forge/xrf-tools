import { SessionId } from "@/core/bindings/types/xrf-app";
import { ISessionIdentity } from "@/core/ipc/session/session.types";
import { Nullable } from "@/lib/types/general";

/**
 * Owns native openings through completion and teardown, including cancelled frontend flows.
 */
export class Session {
  private readonly owned: Set<SessionId> = new Set();
  private readonly pending: Set<SessionId> = new Set();

  private revision: number = 0;

  public constructor(private readonly release: (sessionIds: Array<SessionId>) => Promise<unknown>) {}

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

    this.owned.add(sessionId);
    this.pending.add(sessionId);

    let published: boolean = false;

    try {
      const response: T = await command(sessionId, ...args);

      published = true;

      if (revision !== this.revision) {
        // A close may have finished before this publication reached us; retain it if the second release fails.
        this.owned.add(sessionId);
        await this.release([sessionId]);

        this.owned.delete(sessionId);
      } else {
        // Successful publication replaced the committed value; unfinished calls retain their own identities.
        for (const id of this.owned) {
          if (id !== sessionId && !this.pending.has(id)) {
            this.owned.delete(id);
          }
        }
      }

      return response;
    } catch (error) {
      if (!published) {
        this.owned.delete(sessionId);
      }

      throw error;
    } finally {
      this.pending.delete(sessionId);
    }
  }

  /**
   * Releases owned openings. Failed releases remain retryable.
   *
   * @param restoredId - Identity restored by this view, if any.
   */
  public async close(restoredId?: Nullable<SessionId>): Promise<void> {
    this.revision += 1;

    if (restoredId) {
      this.owned.add(restoredId);
    }

    const ids: Array<SessionId> = [...this.owned];

    if (ids.length === 0) {
      return;
    }

    await this.release(ids);

    for (const id of ids) {
      this.owned.delete(id);
    }
  }
}
