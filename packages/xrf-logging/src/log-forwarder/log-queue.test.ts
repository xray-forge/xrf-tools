import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { ILogRecord } from "#/log-forwarder/log-forwarder.types";
import { LogLevelFilter } from "#/log-forwarder/log-level-filter";
import { LogQueue } from "#/log-forwarder/log-queue";
import { ILogBatch } from "#/log-forwarder/log-queue.types";
import { ELogLevel } from "#/logger.types";

function createRecord(index: number, level: ELogLevel = ELogLevel.INFO): ILogRecord {
  return { level, scope: "scope", time: 1_000 + index, message: `line ${index}` };
}

/** A queue bounded by `records` alone, its byte bound out of reach. */
function createQueue(records: number, bytes: number = Number.MAX_SAFE_INTEGER): LogQueue {
  return new LogQueue({ records, bytes }, "forwarder");
}

function fill(queue: LogQueue, count: number, from: number = 0): void {
  for (let index: number = from; index < from + count; index += 1) {
    queue.push(createRecord(index));
  }
}

function takeAll(queue: LogQueue): ILogBatch {
  return queue.take(Number.MAX_SAFE_INTEGER) as ILogBatch;
}

describe("LogQueue", () => {
  it("keeps the newest records in order at capacity, dropping one for each record past it", () => {
    const queue: LogQueue = createQueue(3);

    fill(queue, 3);
    expect(queue).toHaveLength(3);

    fill(queue, 1_000, 3);

    expect(queue).toHaveLength(3);
    expect(queue.firstWaiting).toBe(1_001);
    expect(queue.lastWritten).toBe(1_003);

    const batch: ILogBatch = takeAll(queue);

    expect(batch.records.map((it: ILogRecord) => it.message)).toEqual([
      "1000 records dropped: the queue overflowed or a send failed",
      "line 1000",
      "line 1001",
      "line 1002",
    ]);
    expect(batch.firstSequence).toBe(1_001);
    expect(batch.lostOnFailure).toBe(1_003);
  });

  it("stamps the warning with the oldest dropped record's time, not later than the next record's", () => {
    const queue: LogQueue = createQueue(2);

    fill(queue, 5);

    const [warning, next]: Array<ILogRecord> = takeAll(queue).records;

    expect(warning).toEqual({
      level: ELogLevel.WARN,
      scope: "forwarder",
      time: 1_000,
      message: "3 records dropped: the queue overflowed or a send failed",
    });
    expect(warning.time).toBeLessThanOrEqual(next.time);
  });

  it("stamps the warning for a lost batch with its oldest record's time", () => {
    const queue: LogQueue = createQueue(10);

    fill(queue, 2);

    const lost: ILogBatch = takeAll(queue);

    fill(queue, 1, 2);
    queue.countLost(lost);

    const [warning, next]: Array<ILogRecord> = takeAll(queue).records;

    expect(warning.message).toBe("2 records dropped: the queue overflowed or a send failed");
    expect(warning.time).toBe(1_000);
    expect(warning.time).toBeLessThanOrEqual(next.time);
  });

  it("takes batches in order, counting the warning against the batch size", () => {
    const queue: LogQueue = createQueue(4);

    fill(queue, 6);

    const first: Nullable<ILogBatch> = queue.take(2);
    const second: Nullable<ILogBatch> = queue.take(2);

    expect(first?.records.map((it: ILogRecord) => it.message)).toEqual([
      "2 records dropped: the queue overflowed or a send failed",
      "line 2",
    ]);
    expect(second?.records.map((it: ILogRecord) => it.message)).toEqual(["line 3", "line 4"]);
    expect(queue.firstWaiting).toBe(6);
    expect(takeAll(queue).records.map((it: ILogRecord) => it.message)).toEqual(["line 5"]);
    expect(queue.take(2)).toBeNull();
    expect(queue.firstWaiting).toBe(7);
  });

  it("takes the warning alone into a batch of one, the records following one a batch", () => {
    const queue: LogQueue = createQueue(2);

    fill(queue, 3);

    const warning: Nullable<ILogBatch> = queue.take(1);

    expect(warning).toEqual({
      records: [expect.objectContaining({ message: "1 record dropped: the queue overflowed or a send failed" })],
      firstSequence: 2,
      queued: 0,
      lostOnFailure: 1,
    });
    expect(queue.take(1)?.records.map((it: ILogRecord) => it.message)).toEqual(["line 1"]);
    expect(queue.take(1)?.records.map((it: ILogRecord) => it.message)).toEqual(["line 2"]);
    expect(queue.take(1)).toBeNull();
  });

  it("keeps the records a filter passes and names the most severe one waiting", () => {
    const queue: LogQueue = createQueue(10);
    const filter: LogLevelFilter = new LogLevelFilter();

    queue.push(createRecord(0, ELogLevel.DEBUG));
    queue.push(createRecord(1, ELogLevel.WARN));
    queue.push(createRecord(2, ELogLevel.INFO));

    expect(queue.mostSevere).toBe(ELogLevel.WARN);

    filter.set(ELogLevel.INFO);
    queue.keepPassing(filter);

    expect(takeAll(queue).records.map((it: ILogRecord) => it.message)).toEqual(["line 1", "line 2"]);
    expect(queue.mostSevere).toBeNull();
  });

  it("keeps the dropped count when the filter empties the queue, and takes its warning alone", () => {
    const queue: LogQueue = createQueue(2);
    const filter: LogLevelFilter = new LogLevelFilter();

    queue.push(createRecord(0, ELogLevel.DEBUG));
    queue.push(createRecord(1, ELogLevel.DEBUG));
    queue.push(createRecord(2, ELogLevel.DEBUG));

    filter.set(ELogLevel.INFO);
    queue.keepPassing(filter);

    expect(queue).toHaveLength(0);
    expect(queue.isEmpty).toBe(false);
    expect(takeAll(queue)).toEqual({
      records: [expect.objectContaining({ message: "1 record dropped: the queue overflowed or a send failed" })],
      firstSequence: 4,
      queued: 0,
      lostOnFailure: 1,
    });
    expect(queue.isEmpty).toBe(true);
    expect(queue.take(1)).toBeNull();
  });

  it("forgets the dropped count where the filter would remove its warning", () => {
    const queue: LogQueue = createQueue(1);
    const filter: LogLevelFilter = new LogLevelFilter();

    fill(queue, 2);
    filter.set(ELogLevel.ERROR);
    queue.keepPassing(filter);

    expect(queue.isEmpty).toBe(true);
    expect(queue.take(1)).toBeNull();
  });

  it("drops the oldest records while their text is over the byte bound, keeping the newest however long", () => {
    // "line N" is six UTF-16 units, twelve bytes: three fit in 40 bytes, a fourth does not.
    const queue: LogQueue = createQueue(10, 40);

    fill(queue, 4);

    expect(queue).toHaveLength(3);
    expect(takeAll(queue).records.map((it: ILogRecord) => it.message)).toEqual([
      "1 record dropped: the queue overflowed or a send failed",
      "line 1",
      "line 2",
      "line 3",
    ]);

    queue.push({ ...createRecord(4), message: "x".repeat(100) });

    expect(queue).toHaveLength(1);
  });

  it("frees the bytes of the records taken and filtered, so later records fit again", () => {
    const queue: LogQueue = createQueue(10, 40);
    const filter: LogLevelFilter = new LogLevelFilter();

    fill(queue, 3);
    queue.take(2);
    queue.push(createRecord(3, ELogLevel.DEBUG));
    filter.set(ELogLevel.INFO);
    queue.keepPassing(filter);
    fill(queue, 2, 4);

    expect(takeAll(queue).records.map((it: ILogRecord) => it.message)).toEqual(["line 2", "line 4", "line 5"]);
  });

  it("does not count again a lost batch that carried only the warning", () => {
    const queue: LogQueue = createQueue(10);

    fill(queue, 2);
    queue.countLost(takeAll(queue));

    const warning: ILogBatch = takeAll(queue);

    expect(warning.queued).toBe(0);

    queue.countLost(warning);

    expect(queue.isEmpty).toBe(true);
  });
});
