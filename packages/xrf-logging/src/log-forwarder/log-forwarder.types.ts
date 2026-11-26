import { ELogLevel } from "#/logger.types";

/**
 * One logged line as it is forwarded: serialized and stamped when it was logged, since what it logged may change
 * before it is sent.
 */
export interface ILogRecord {
  level: ELogLevel;
  /** The prefix of the logger that wrote it. */
  scope: string;
  /** Whole milliseconds since the Unix epoch at which it was logged. */
  time: number;
  /** Its values written as one text, capped in length. */
  message: string;
}

/**
 * Sends one batch of records, in their order, to wherever they are kept.
 *
 * @param records - At most the forwarder's batch size, oldest first.
 * @returns Settles once the batch is kept; a rejection means it was lost.
 */
export type TLogBatchSender = (records: Array<ILogRecord>) => Promise<unknown>;

/** The numbers that bound what a forwarder holds and how often it sends. */
export interface ILogForwarderLimits {
  /** The most records one send carries, a warning about dropped records included; no more than the receiver takes. */
  batchSize: number;
  /** The most records waiting to be sent; past it the oldest are dropped. */
  queueSize: number;
  /**
   * The most bytes of text waiting to be sent, counted as the page holds it (two a UTF-16 unit); past it the oldest
   * records are dropped, so a burst of long records cannot hold what a count alone would let it.
   */
  queueBytes: number;
  /** Milliseconds a line waits for its batch, and the least time between two sends a warning hurries. */
  flushInterval: number;
  /** The most characters one record's text keeps before it is cut and marked as cut. */
  messageLength: number;
}
