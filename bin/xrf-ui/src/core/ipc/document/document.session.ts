import { IDocumentSession } from "@/core/ipc/document/document.types";
import { Nullable } from "@/lib/types/general";

/** Owns native openings until they settle or are explicitly released, including cancelled frontend flows. */
export class DocumentSession {
  private readonly owned: Set<string> = new Set();
  private readonly pending: Set<string> = new Set();

  private revision: number = 0;

  public constructor(private readonly release: (sessionIds: Array<string>) => Promise<unknown>) {}

  /** Allocates the identity before dispatch, so teardown can invalidate an opening that has not answered yet. */
  public async open<T extends IDocumentSession>(open: (sessionId: string) => Promise<T>): Promise<T> {
    const sessionId: string = crypto.randomUUID();
    const revision: number = ++this.revision;

    this.owned.add(sessionId);
    this.pending.add(sessionId);

    let published: boolean = false;

    try {
      const response: T = await open(sessionId);

      published = true;

      if (revision !== this.revision) {
        // A close may have finished before this publication reached us; retain it if the second release fails.
        this.owned.add(sessionId);
        await this.release([sessionId]);

        this.owned.delete(sessionId);
      } else {
        // Successful publication replaced older committed documents; unfinished calls retain their own identities.
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

  /** Releases this lifetime's openings and the document it restored. Failed releases remain retryable. */
  public async close(restoredId?: Nullable<string>): Promise<void> {
    this.revision += 1;

    if (restoredId) {
      this.owned.add(restoredId);
    }

    const ids: Array<string> = [...this.owned];

    if (ids.length === 0) {
      return;
    }

    await this.release(ids);

    for (const id of ids) {
      this.owned.delete(id);
    }
  }
}
