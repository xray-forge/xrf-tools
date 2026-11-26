import { Nullable } from "@xrf/types";

import { LogFlushSchedule } from "#/log-forwarder/log-flush-schedule";
import { LogFlushWaiters } from "#/log-forwarder/log-flush-waiters";
import { ILogForwarderLimits, TLogBatchSender } from "#/log-forwarder/log-forwarder.types";
import { LogLevelFilter } from "#/log-forwarder/log-level-filter";
import { LogQueue } from "#/log-forwarder/log-queue";
import { ILogBatch } from "#/log-forwarder/log-queue.types";
import { LOG_LEVEL_RANK } from "#/log-level-rank";
import { LogTextWriter } from "#/log-text/log-text-writer";
import { ELogLevel, ILogSink } from "#/logger.types";

/**
 * Forwards logged lines in batches to a sender, a cheap sink for every logger to write into. Each line becomes a record
 * as it is written, serialized and stamped, and waits in a bounded queue; one send is in flight at a time.
 *
 * Until {@link LogForwarder.setLevel} is first called every line is kept and nothing is sent, so the lines logged before
 * the level is known are filtered and sent once it is. A batch goes when the flush interval passes, when a full batch
 * waits, at once for a warning or an error but at most once per interval, or on {@link LogForwarder.flush}. Past the
 * queue's bound the oldest records are dropped, and the next batch opens with a warning counting them; with nothing else
 * waiting, that warning is sent alone at the interval. A count is lost only where the warning's own send fails, which
 * goes to the console like any failed send, or where the level filters warnings out.
 */
export class LogForwarder implements ILogSink {
  /** The limits used where the constructor is given none. */
  public static readonly DEFAULT_LIMITS: Readonly<ILogForwarderLimits> = {
    batchSize: 512,
    queueSize: 8_192,
    queueBytes: 8 * 1_024 * 1_024,
    flushInterval: 250,
    messageLength: LogTextWriter.MAX_LENGTH,
  };

  /** The scope of the forwarder's own lines: the warning saying how many records were dropped, and a failed send. */
  public static readonly SCOPE: string = "log-forwarder";

  /**
   * Whether a line just queued hurries a send rather than waiting for the interval: a warning or an error does.
   *
   * @param level - The line's level.
   */
  private static isUrgent(level: ELogLevel): boolean {
    return LOG_LEVEL_RANK[level] <= LOG_LEVEL_RANK[ELogLevel.WARN];
  }

  private readonly send: TLogBatchSender;
  private readonly limits: ILogForwarderLimits;
  private readonly queue: LogQueue;
  private readonly schedule: LogFlushSchedule;
  private readonly filter: LogLevelFilter = new LogLevelFilter();
  private readonly waiters: LogFlushWaiters = new LogFlushWaiters();

  /** The number of the first record in the send in flight, or null when none is. */
  private sendingFrom: Nullable<number> = null;

  /**
   * @param send - Sends one batch; the forwarder awaits it before sending the next. It must not log through a
   *   `Logger`, whose line would come back to this forwarder.
   * @param limits - Overrides {@link LogForwarder.DEFAULT_LIMITS}.
   * @throws {RangeError} Where the batch size is below one, since such a batch could carry nothing.
   */
  public constructor(send: TLogBatchSender, limits: Partial<ILogForwarderLimits> = {}) {
    this.send = send;
    this.limits = { ...LogForwarder.DEFAULT_LIMITS, ...limits };

    if (!(this.limits.batchSize >= 1)) {
      throw new RangeError(`A log batch must carry at least one record, not ${this.limits.batchSize}`);
    }

    this.queue = new LogQueue({ records: this.limits.queueSize, bytes: this.limits.queueBytes }, LogForwarder.SCOPE);
    this.schedule = new LogFlushSchedule(this.limits.flushInterval, () => this.sendNextBatch());
  }

  public write(level: ELogLevel, scope: string, args: ReadonlyArray<unknown>): void {
    if (!this.filter.passes(level)) {
      return;
    }

    this.queue.push({ level, scope, time: Date.now(), message: LogTextWriter.write(args, this.limits.messageLength) });
    this.scheduleSend(level);
  }

  /**
   * Sets the most verbose level kept, removing what is queued below it, which is filtered rather than counted as
   * dropped, and sending what is left.
   *
   * @param level - The most verbose level kept, or null to keep nothing, from the queue or from now on.
   */
  public setLevel(level: Nullable<ELogLevel>): void {
    this.filter.set(level);
    this.queue.keepPassing(this.filter);
    this.scheduleSend(this.queue.mostSevere);
  }

  /**
   * Sends everything queued now, without waiting for the flush interval.
   *
   * @returns Settles once every record queued at the call has been sent or dropped; before the level is known, that
   *   waits for it.
   */
  public flush(): Promise<void> {
    const flushed: Promise<void> = this.waiters.wait(this.queue.lastWritten);

    if (this.filter.isKnown) {
      this.schedule.markDue();
    }

    this.scheduleSend(null);

    return flushed;
  }

  /**
   * Settles the flushes whose records have left, then decides what happens next: nothing before the level is known,
   * otherwise a send when one is due and none is in flight, the interval while anything waits, or a rest.
   *
   * @param queued - The most severe level just queued, which hurries a send when urgent; null when none was.
   */
  private scheduleSend(queued: Nullable<ELogLevel>): void {
    // Every record before the oldest still pending has left; the send in flight holds that one, else the queue does.
    this.waiters.settle((this.sendingFrom ?? this.queue.firstWaiting) - 1);

    if (!this.filter.isKnown) {
      return;
    }

    if (this.queue.isEmpty) {
      return this.schedule.reset();
    }

    if (queued !== null && LogForwarder.isUrgent(queued)) {
      this.schedule.hurry();
    }

    if (this.queue.length >= this.limits.batchSize) {
      this.schedule.markDue();
    }

    if (this.schedule.isDue) {
      this.sendNextBatch();
    } else {
      this.schedule.wait();
    }
  }

  /** Sends the next batch when one is due and none is in flight. */
  private sendNextBatch(): void {
    if (this.sendingFrom !== null || !this.schedule.isDue) {
      return;
    }

    this.schedule.reset();

    const batch: Nullable<ILogBatch> = this.queue.take(this.limits.batchSize);

    if (batch === null) {
      return;
    }

    this.sendingFrom = batch.firstSequence;

    // What a due send could not carry was due as well, so it follows as soon as this one settles.
    if (this.queue.length > 0) {
      this.schedule.markDue();
    }

    let sending: Promise<unknown>;

    try {
      sending = this.send(batch.records);
    } catch (error: unknown) {
      sending = Promise.reject(error);
    }

    sending.then(
      () => this.onSettled(),
      (error: unknown) => {
        this.queue.countLost(batch);
        console.error(`[${LogForwarder.SCOPE}] Cannot send ${batch.records.length} records, dropping them:`, error);
        this.onSettled();
      }
    );
  }

  private onSettled(): void {
    this.sendingFrom = null;
    this.scheduleSend(null);
  }
}
