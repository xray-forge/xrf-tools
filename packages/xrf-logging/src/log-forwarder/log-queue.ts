import { Nullable } from "@xrf/types";

import { ILogRecord } from "#/log-forwarder/log-forwarder.types";
import { LogLevelFilter } from "#/log-forwarder/log-level-filter";
import { ILogBatch, ILogQueueBound, IQueuedLogRecord } from "#/log-forwarder/log-queue.types";
import { LOG_LEVEL_RANK } from "#/log-level-rank";
import { ELogLevel } from "#/logger.types";

/**
 * The records waiting to be sent, oldest first, numbered in the order they were written.
 *
 * Bounded by count and by the bytes of their text: past either the oldest records are dropped and counted, and the next
 * batch taken opens with a warning saying how many were dropped. Dropping the oldest takes amortized constant time,
 * however full the queue is.
 */
export class LogQueue {
  /**
   * The bytes a record's text takes as the page holds it, two a UTF-16 unit.
   *
   * @param record - The record.
   */
  private static weigh(record: ILogRecord): number {
    return record.message.length * 2;
  }

  private readonly bound: ILogQueueBound;
  private readonly scope: string;

  /** The records from {@link head} on are waiting; those before it were taken or dropped, kept until compacting pays. */
  private entries: Array<IQueuedLogRecord> = [];
  /** The index of the oldest record still waiting. */
  private head: number = 0;
  /** The bytes of the waiting records' text. */
  private bytes: number = 0;
  private lastSequence: number = 0;
  /** How many records were dropped since the last batch. */
  private dropped: number = 0;
  /** When the oldest of the dropped records was logged, or infinity while none is counted. */
  private droppedSince: number = Infinity;

  /**
   * @param bound - How much is kept waiting.
   * @param scope - The scope of the warning saying how many records were dropped.
   */
  public constructor(bound: ILogQueueBound, scope: string) {
    this.bound = bound;
    this.scope = scope;
  }

  /** How many records are waiting. */
  public get length(): number {
    return this.entries.length - this.head;
  }

  /** Whether nothing is left to send: no record waits and no dropped record is still to be reported. */
  public get isEmpty(): boolean {
    return this.length === 0 && this.dropped === 0;
  }

  /** The number of the last record written, sent or not. */
  public get lastWritten(): number {
    return this.lastSequence;
  }

  /** The number of the oldest record still waiting, or the next to be written when none is. */
  public get firstWaiting(): number {
    return this.length > 0 ? this.entries[this.head].sequence : this.lastSequence + 1;
  }

  /** The most severe level waiting, or null when nothing is. */
  public get mostSevere(): Nullable<ELogLevel> {
    let level: Nullable<ELogLevel> = null;

    for (let index: number = this.head; index < this.entries.length; index += 1) {
      const next: ELogLevel = this.entries[index].record.level;

      if (level === null || LOG_LEVEL_RANK[next] < LOG_LEVEL_RANK[level]) {
        level = next;
      }
    }

    return level;
  }

  /**
   * Adds a record, dropping the oldest while the queue is over its bound. The newest is always kept: a message is cut
   * far below any byte bound the forwarder sets.
   *
   * @param record - The record to send.
   */
  public push(record: ILogRecord): void {
    this.lastSequence += 1;
    this.entries.push({ sequence: this.lastSequence, record });
    this.bytes += LogQueue.weigh(record);

    while (this.length > 1 && (this.length > this.bound.records || this.bytes > this.bound.bytes)) {
      const oldest: ILogRecord = this.entries[this.head].record;

      this.countDropped(1, oldest.time);
      this.bytes -= LogQueue.weigh(oldest);
      this.head += 1;
    }

    this.compact();
  }

  /**
   * Removes the records a filter does not pass, which are filtered rather than dropped. The count of dropped records is
   * kept for its warning, unless the filter would remove that warning too.
   *
   * @param filter - The levels kept.
   */
  public keepPassing(filter: LogLevelFilter): void {
    this.entries = this.entries.filter(
      (it: IQueuedLogRecord, index: number) => index >= this.head && filter.passes(it.record.level)
    );
    this.head = 0;
    this.bytes = this.entries.reduce((sum: number, it: IQueuedLogRecord) => sum + LogQueue.weigh(it.record), 0);

    if (!filter.passes(ELogLevel.WARN)) {
      this.dropped = 0;
      this.droppedSince = Infinity;
    }
  }

  /**
   * Takes the oldest records to send together. The warning about dropped records takes a place of its own, so a batch
   * of one carries it alone and the records follow in the next; with no record waiting, the warning goes alone.
   *
   * @param size - The most records the batch carries, the warning about dropped records included; at least one.
   * @returns The batch, or null when nothing is left to send.
   */
  public take(size: number): Nullable<ILogBatch> {
    if (this.isEmpty) {
      return null;
    }

    const carried: number = this.dropped;
    const firstSequence: number = this.firstWaiting;
    const end: number = Math.min(this.head + size - (carried > 0 ? 1 : 0), this.entries.length);
    const taken: Array<IQueuedLogRecord> = this.entries.slice(this.head, end);
    const records: Array<ILogRecord> = taken.map((it: IQueuedLogRecord) => it.record);

    if (carried > 0) {
      records.unshift(this.describeDropped(carried));
    }

    this.head = end;
    this.bytes -= taken.reduce((sum: number, it: IQueuedLogRecord) => sum + LogQueue.weigh(it.record), 0);
    this.dropped = 0;
    this.droppedSince = Infinity;
    this.compact();

    return { records, firstSequence, queued: taken.length, lostOnFailure: carried + taken.length };
  }

  /**
   * Counts a batch whose send failed as dropped, so the next batch's warning includes it. A lost batch that carried
   * only the warning is not counted again: a sender failing for good would otherwise be sent that warning forever.
   *
   * @param batch - The batch that was lost.
   */
  public countLost(batch: ILogBatch): void {
    if (batch.queued === 0) {
      return;
    }

    this.countDropped(batch.lostOnFailure, batch.records[0].time);
  }

  /**
   * Adds dropped records to the count the next batch's warning reports.
   *
   * @param count - How many records were dropped.
   * @param since - When the oldest of them was logged.
   */
  private countDropped(count: number, since: number): void {
    this.dropped += count;
    this.droppedSince = Math.min(this.droppedSince, since);
  }

  /**
   * The warning a batch opens with, saying how many records were dropped. It is stamped with the oldest dropped record's
   * time, so what receives the batch stays in stamp order: every record still waiting was logged after it.
   *
   * @param count - How many records were dropped since the last batch.
   */
  private describeDropped(count: number): ILogRecord {
    return {
      level: ELogLevel.WARN,
      scope: this.scope,
      time: this.droppedSince,
      message: `${count} ${count === 1 ? "record" : "records"} dropped: the queue overflowed or a send failed`,
    };
  }

  /** Lets go of the records before the head once they outnumber those waiting, which keeps each drop's cost constant. */
  private compact(): void {
    if (this.head * 2 >= this.entries.length) {
      this.entries = this.entries.slice(this.head);
      this.head = 0;
    }
  }
}
