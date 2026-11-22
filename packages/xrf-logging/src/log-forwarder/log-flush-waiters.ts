import { ILogFlushWaiter } from "#/log-forwarder/log-flush-waiters.types";

/**
 * The flushes waiting for records to leave, each settled once every record up to the one it waits for has.
 */
export class LogFlushWaiters {
  private waiters: Array<ILogFlushWaiter> = [];

  /**
   * @param sequence - The number of the last record to wait for.
   * @returns Settles once {@link settle} is told that record is done.
   */
  public wait(sequence: number): Promise<void> {
    return new Promise((resolve: () => void) => {
      this.waiters.push({ sequence, resolve });
    });
  }

  /**
   * Settles each flush whose records are all done.
   *
   * @param doneThrough - The number of the last record such that it and every record before it left.
   */
  public settle(doneThrough: number): void {
    if (this.waiters.length === 0) {
      return;
    }

    const pending: Array<ILogFlushWaiter> = [];

    for (const waiter of this.waiters) {
      if (waiter.sequence <= doneThrough) {
        waiter.resolve();
      } else {
        pending.push(waiter);
      }
    }

    this.waiters = pending;
  }
}
