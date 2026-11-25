import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";

import { LogForwarder } from "#/log-forwarder/log-forwarder";
import { ILogForwarderLimits, ILogRecord } from "#/log-forwarder/log-forwarder.types";
import { ELogLevel } from "#/logger.types";

interface ISentBatch {
  records: Array<ILogRecord>;
  resolve: () => void;
  reject: (error: unknown) => void;
}

interface IForwarderHarness {
  forwarder: LogForwarder;
  sent: Array<ISentBatch>;
}

function createForwarder(limits: Partial<ILogForwarderLimits> = {}): IForwarderHarness {
  const sent: Array<ISentBatch> = [];

  function send(records: Array<ILogRecord>): Promise<unknown> {
    return new Promise((resolve: (value: unknown) => void, reject: (error: unknown) => void) => {
      sent.push({ records, resolve: () => resolve(null), reject });
    });
  }

  return { forwarder: new LogForwarder(send, limits), sent };
}

function messagesOf(batch: ISentBatch): Array<string> {
  return batch.records.map((record: ILogRecord) => record.message);
}

function writeInfo(forwarder: LogForwarder, count: number, from: number = 0): void {
  for (let index: number = from; index < from + count; index += 1) {
    forwarder.write(ELogLevel.INFO, "scope", [`line ${index}`]);
  }
}

async function settle(): Promise<void> {
  await jest.advanceTimersByTimeAsync(0);
}

describe("LogForwarder", () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
    jest.restoreAllMocks();
  });

  it("sends a burst of info lines as one batch once the interval passes", async () => {
    const { forwarder, sent } = createForwarder();

    forwarder.setLevel(ELogLevel.DEBUG);
    writeInfo(forwarder, 3);

    await jest.advanceTimersByTimeAsync(249);
    expect(sent).toHaveLength(0);

    await jest.advanceTimersByTimeAsync(1);
    expect(sent).toHaveLength(1);
    expect(messagesOf(sent[0])).toEqual(["line 0", "line 1", "line 2"]);
  });

  it("stamps and serializes each line as it is written", async () => {
    const { forwarder, sent } = createForwarder();
    const state: { count: number } = { count: 1 };

    jest.setSystemTime(1_700_000_000_000);
    forwarder.setLevel(ELogLevel.DEBUG);
    forwarder.write(ELogLevel.INFO, "scope", ["state", state]);
    state.count = 2;

    await jest.advanceTimersByTimeAsync(250);

    expect(sent[0].records).toEqual([
      { level: ELogLevel.INFO, scope: "scope", time: 1_700_000_000_000, message: 'state {"count":1}' },
    ]);
  });

  it("sends a warning at once, carrying the lines queued before it in order", async () => {
    const { forwarder, sent } = createForwarder();

    forwarder.setLevel(ELogLevel.DEBUG);
    writeInfo(forwarder, 2);
    forwarder.write(ELogLevel.WARN, "scope", ["slow"]);

    expect(sent).toHaveLength(1);
    expect(messagesOf(sent[0])).toEqual(["line 0", "line 1", "slow"]);
  });

  it("holds a second warning inside the interval for the timer", async () => {
    const { forwarder, sent } = createForwarder();

    forwarder.setLevel(ELogLevel.DEBUG);
    forwarder.write(ELogLevel.ERROR, "scope", ["first"]);
    sent[0].resolve();
    await settle();

    await jest.advanceTimersByTimeAsync(100);
    forwarder.write(ELogLevel.WARN, "scope", ["second"]);

    expect(sent).toHaveLength(1);

    await jest.advanceTimersByTimeAsync(250);

    expect(sent).toHaveLength(2);
    expect(messagesOf(sent[1])).toEqual(["second"]);
  });

  it("keeps one send in flight, sending what arrived meanwhile once it settles", async () => {
    const { forwarder, sent } = createForwarder();

    forwarder.setLevel(ELogLevel.DEBUG);
    forwarder.write(ELogLevel.ERROR, "scope", ["first"]);
    writeInfo(forwarder, 2);

    await jest.advanceTimersByTimeAsync(1_000);
    expect(sent).toHaveLength(1);

    sent[0].resolve();
    await settle();

    expect(sent).toHaveLength(2);
    expect(messagesOf(sent[1])).toEqual(["line 0", "line 1"]);
  });

  it("never sends more than a batch, sending a full batch without waiting", async () => {
    const { forwarder, sent } = createForwarder({ batchSize: 4 });

    forwarder.setLevel(ELogLevel.DEBUG);
    writeInfo(forwarder, 10);

    expect(sent).toHaveLength(1);
    expect(messagesOf(sent[0])).toEqual(["line 0", "line 1", "line 2", "line 3"]);

    sent[0].resolve();
    await settle();
    sent[1].resolve();
    await settle();

    expect(sent.map((batch: ISentBatch) => batch.records.length)).toEqual([4, 4, 2]);
    expect(messagesOf(sent[2])).toEqual(["line 8", "line 9"]);
  });

  it("neither queues nor sends a line its level filters", async () => {
    const { forwarder, sent } = createForwarder();

    forwarder.setLevel(ELogLevel.INFO);
    forwarder.write(ELogLevel.DEBUG, "scope", ["detail"]);

    await forwarder.flush();
    await jest.advanceTimersByTimeAsync(1_000);

    expect(sent).toHaveLength(0);
  });

  it("keeps lines written before the level is known, then sends those it lets through", async () => {
    const { forwarder, sent } = createForwarder();

    forwarder.write(ELogLevel.DEBUG, "scope", ["detail"]);
    forwarder.write(ELogLevel.INFO, "scope", ["opened"]);
    forwarder.write(ELogLevel.WARN, "scope", ["slow"]);

    await jest.advanceTimersByTimeAsync(1_000);
    expect(sent).toHaveLength(0);

    forwarder.setLevel(ELogLevel.INFO);

    expect(sent).toHaveLength(1);
    expect(messagesOf(sent[0])).toEqual(["opened", "slow"]);
  });

  it("sends nothing and keeps nothing when told there is no level", async () => {
    const { forwarder, sent } = createForwarder();

    forwarder.write(ELogLevel.ERROR, "scope", ["early"]);
    forwarder.setLevel(null);
    forwarder.write(ELogLevel.ERROR, "scope", ["late"]);

    await forwarder.flush();
    await jest.advanceTimersByTimeAsync(1_000);

    expect(sent).toHaveLength(0);
  });

  it("drops the oldest lines past the queue size and opens the next batch saying how many", async () => {
    const { forwarder, sent } = createForwarder({ queueSize: 3 });

    writeInfo(forwarder, 5);
    forwarder.setLevel(ELogLevel.DEBUG);

    await jest.advanceTimersByTimeAsync(250);

    expect(sent[0].records[0]).toEqual({
      level: ELogLevel.WARN,
      scope: LogForwarder.SCOPE,
      time: expect.any(Number),
      message: "2 records dropped: the queue overflowed or a send failed",
    });
    expect(messagesOf(sent[0]).slice(1)).toEqual(["line 2", "line 3", "line 4"]);
  });

  it("stamps the warning about dropped lines no later than the line after it", async () => {
    const { forwarder, sent } = createForwarder({ queueSize: 2 });

    for (let index: number = 0; index < 4; index += 1) {
      jest.setSystemTime(1_700_000_000_000 + index * 10);
      forwarder.write(ELogLevel.INFO, "scope", [`line ${index}`]);
    }

    jest.setSystemTime(1_700_000_001_000);
    forwarder.setLevel(ELogLevel.DEBUG);
    await jest.advanceTimersByTimeAsync(250);

    const [warning, next] = sent[0].records;

    expect(warning.scope).toBe(LogForwarder.SCOPE);
    expect(warning.time).toBe(1_700_000_000_000);
    expect(warning.time).toBeLessThanOrEqual(next.time);
  });

  it("counts the warning against the batch size", async () => {
    const { forwarder, sent } = createForwarder({ batchSize: 2, queueSize: 2 });

    writeInfo(forwarder, 3);
    forwarder.setLevel(ELogLevel.DEBUG);

    expect(messagesOf(sent[0])).toEqual(["1 record dropped: the queue overflowed or a send failed", "line 1"]);
  });

  it("sends the warning alone and then each line where a batch carries one record", async () => {
    const { forwarder, sent } = createForwarder({ batchSize: 1, queueSize: 2 });
    const done: jest.Mock<() => void> = jest.fn();

    writeInfo(forwarder, 3);
    forwarder.setLevel(ELogLevel.DEBUG);
    void forwarder.flush().then(done);

    for (let index: number = 0; index < 3; index += 1) {
      sent[index].resolve();
      await settle();
    }

    expect(sent.map(messagesOf)).toEqual([
      ["1 record dropped: the queue overflowed or a send failed"],
      ["line 1"],
      ["line 2"],
    ]);
    expect(done).toHaveBeenCalledTimes(1);
  });

  it("refuses a batch size that could carry nothing", () => {
    expect(() => createForwarder({ batchSize: 0 })).toThrow(RangeError);
  });

  it("drops and counts a batch whose send fails, reporting it to the console without retrying", async () => {
    const { forwarder, sent } = createForwarder();
    const report: jest.SpiedFunction<typeof console.error> = jest.spyOn(console, "error").mockImplementation(() => {});

    forwarder.setLevel(ELogLevel.DEBUG);
    writeInfo(forwarder, 2);
    await jest.advanceTimersByTimeAsync(250);

    sent[0].reject(new Error("refused"));
    await settle();
    writeInfo(forwarder, 1, 2);
    await jest.advanceTimersByTimeAsync(250);

    expect(report).toHaveBeenCalledTimes(1);
    expect(messagesOf(sent[1])).toEqual(["2 records dropped: the queue overflowed or a send failed", "line 2"]);
  });

  it("sends the count of a failed batch alone when nothing follows it, and gives up if that send fails too", async () => {
    const { forwarder, sent } = createForwarder();
    const report: jest.SpiedFunction<typeof console.error> = jest.spyOn(console, "error").mockImplementation(() => {});

    forwarder.setLevel(ELogLevel.DEBUG);
    writeInfo(forwarder, 2);
    await jest.advanceTimersByTimeAsync(250);

    sent[0].reject(new Error("refused"));
    await jest.advanceTimersByTimeAsync(250);

    expect(sent.map(messagesOf).slice(1)).toEqual([["2 records dropped: the queue overflowed or a send failed"]]);

    sent[1].reject(new Error("refused"));
    await jest.advanceTimersByTimeAsync(1_000);

    expect(sent).toHaveLength(2);
    expect(report).toHaveBeenCalledTimes(2);
  });

  it("sends the count of records dropped before the level is known though the level filters every one left", async () => {
    const { forwarder, sent } = createForwarder({ queueSize: 2 });

    forwarder.write(ELogLevel.INFO, "scope", ["line 0"]);
    forwarder.write(ELogLevel.DEBUG, "scope", ["detail 1"]);
    forwarder.write(ELogLevel.DEBUG, "scope", ["detail 2"]);
    forwarder.setLevel(ELogLevel.INFO);
    await jest.advanceTimersByTimeAsync(250);

    expect(sent.map(messagesOf)).toEqual([["1 record dropped: the queue overflowed or a send failed"]]);
  });

  it("counts the dropped records a failed batch carried again", async () => {
    const { forwarder, sent } = createForwarder({ queueSize: 1 });

    jest.spyOn(console, "error").mockImplementation(() => {});
    writeInfo(forwarder, 2);
    forwarder.setLevel(ELogLevel.DEBUG);
    await jest.advanceTimersByTimeAsync(250);

    sent[0].reject(new Error("refused"));
    await settle();

    writeInfo(forwarder, 1, 2);
    await jest.advanceTimersByTimeAsync(250);

    expect(messagesOf(sent[1])[0]).toBe("2 records dropped: the queue overflowed or a send failed");
  });

  it("treats a sender that throws as a failed send", async () => {
    const report: jest.SpiedFunction<typeof console.error> = jest.spyOn(console, "error").mockImplementation(() => {});
    const forwarder: LogForwarder = new LogForwarder(() => {
      throw new Error("no transport");
    });

    forwarder.setLevel(ELogLevel.DEBUG);
    forwarder.write(ELogLevel.ERROR, "scope", ["failed"]);

    await expect(forwarder.flush()).resolves.toBeUndefined();
    expect(report).toHaveBeenCalledTimes(1);
  });

  it("settles a flush once everything queued at the call has been sent", async () => {
    const { forwarder, sent } = createForwarder({ batchSize: 2 });
    const done: jest.Mock<() => void> = jest.fn();

    forwarder.setLevel(ELogLevel.DEBUG);
    writeInfo(forwarder, 3);
    void forwarder.flush().then(done);

    expect(sent).toHaveLength(1);

    sent[0].resolve();
    await settle();

    expect(done).not.toHaveBeenCalled();
    expect(messagesOf(sent[1])).toEqual(["line 2"]);

    writeInfo(forwarder, 1, 3);
    sent[1].resolve();
    await settle();

    expect(done).toHaveBeenCalledTimes(1);
  });

  it("settles a flush at once when nothing is queued", async () => {
    const { forwarder } = createForwarder();

    forwarder.setLevel(ELogLevel.DEBUG);

    await expect(forwarder.flush()).resolves.toBeUndefined();
  });
});
