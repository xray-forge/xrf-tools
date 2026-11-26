import { ILogRecord } from "#/log-forwarder/log-forwarder.types";

/** How much the queue keeps waiting before it drops its oldest records. */
export interface ILogQueueBound {
  /** The most records. */
  records: number;
  /** The most bytes of their text, two a UTF-16 unit. */
  bytes: number;
}

/** A record waiting in the queue, numbered in the order it was written. */
export interface IQueuedLogRecord {
  sequence: number;
  record: ILogRecord;
}

/** Records taken from the queue to be sent together. */
export interface ILogBatch {
  /** Oldest first, opening with a warning when records were dropped since the last batch. */
  records: Array<ILogRecord>;
  /** The number of the oldest queued record not sent before it: the first it carries, or the next after its warning. */
  firstSequence: number;
  /** How many of the queue's records it carries, its warning apart. */
  queued: number;
  /** How many records are lost should its send fail: those it carries and those its warning counts. */
  lostOnFailure: number;
}
