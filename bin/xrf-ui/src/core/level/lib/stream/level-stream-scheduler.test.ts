import { describe, expect, it } from "@jest/globals";
import { Maybe } from "@xrf/types";

import { ILevelStreamSchedulerHost, LevelStreamScheduler } from "@/core/level/lib/stream/level-stream-scheduler";

/** A read that is finished when the test says so, which is how concurrency is observable at all. */
interface IHeldRead {
  sector: number;
  finish: () => void;
}

/** A host whose reads are held open, recording what it was asked for and what it was told. */
function mockHost(resident: Array<number> = []): {
  host: ILevelStreamSchedulerHost;
  reads: Array<IHeldRead>;
  progress: Array<string>;
  settled: () => number;
  /** Finishes one held read, as a sector arriving would. */
  arrive: (sector: number) => Promise<void>;
} {
  const held: Set<number> = new Set(resident);
  const reads: Array<IHeldRead> = [];
  const progress: Array<string> = [];

  let settlements: number = 0;

  return {
    arrive: async (sector: number): Promise<void> => {
      const read: Maybe<IHeldRead> = reads.find((it: IHeldRead) => it.sector === sector);

      held.add(sector);
      read?.finish();

      // Two turns: one for the read's own promise, one for the continuation the scheduler queued behind it.
      await Promise.resolve();
      await Promise.resolve();
    },
    host: {
      isResident: (sector: number): boolean => held.has(sector),
      read: (sector: number): Promise<void> =>
        new Promise<void>((resolve: () => void) => {
          reads.push({ finish: resolve, sector });
        }),
      report: (loaded: number, total: number): void => {
        progress.push(`${loaded}/${total}`);
      },
      settle: (): void => {
        settlements += 1;
      },
    },
    progress,
    reads,
    settled: (): number => settlements,
  };
}

/** Sectors the host has been asked for, in the order it was asked. */
function asked(reads: Array<IHeldRead>): Array<number> {
  return reads.map((it: IHeldRead) => it.sector);
}

describe("LevelStreamScheduler", () => {
  it("keeps only as many reads in flight as it is allowed", () => {
    const { host, reads } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    scheduler.setConcurrency(2);
    void scheduler.setTarget([0, 1, 2, 3]);

    expect(asked(reads)).toEqual([0, 1]);
    expect(scheduler.reading).toBe(2);
  });

  it("starts the next sector as each one arrives, nearest first", async () => {
    const { host, reads, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    scheduler.setConcurrency(2);
    void scheduler.setTarget([0, 1, 2, 3]);

    await arrive(0);

    expect(asked(reads)).toEqual([0, 1, 2]);

    await arrive(1);

    expect(asked(reads)).toEqual([0, 1, 2, 3]);
  });

  it("never reads a sector that is already held", () => {
    const { host, reads } = mockHost([1]);
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    void scheduler.setTarget([0, 1, 2]);

    expect(asked(reads)).toEqual([0, 2]);
  });

  // The defect the scheduler exists for. Under boost the camera reported every frame, and every report cancelled
  // the plan in flight - so the queue grew faster than it drained and the level never finished arriving.
  it("does not restart a read because the camera reported again", () => {
    const { host, reads } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    scheduler.setConcurrency(1);
    void scheduler.setTarget([0, 1]);
    void scheduler.setTarget([0, 1]);
    void scheduler.setTarget([0, 1]);

    expect(asked(reads)).toEqual([0]);
  });

  // The other half: a report that changes nothing must not run the level's housekeeping, because that reads every
  // resident sector's surfaces and a report lands on every frame of a flight.
  it("settles nothing when a report changes nothing", () => {
    const { host, settled, progress } = mockHost([0, 1]);
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    void scheduler.setTarget([0, 1]);
    void scheduler.setTarget([0, 1]);

    expect(settled()).toBe(0);
    expect(progress).toEqual([]);
  });

  it("settles once when everything wanted has arrived", async () => {
    const { host, settled, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    void scheduler.setTarget([0, 1]);

    await arrive(0);

    expect(settled()).toBe(0);

    await arrive(1);

    expect(settled()).toBe(1);
  });

  it("drops a queued sector the camera has moved away from", async () => {
    const { host, reads, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    scheduler.setConcurrency(1);
    void scheduler.setTarget([0, 1, 2]);
    void scheduler.setTarget([0, 5]);

    await arrive(0);

    // One and two were queued and are wanted no longer; five is, so five is what follows.
    expect(asked(reads)).toEqual([0, 5]);
  });

  // A read already started cannot be called back, so it finishes - but nothing waits on it, and the plan that
  // dropped it is the one that decides whether to keep what it brings.
  it("does not wait for a read the camera has moved away from", async () => {
    const { host, settled, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    scheduler.setConcurrency(2);
    void scheduler.setTarget([0, 1]);
    void scheduler.setTarget([1]);

    await arrive(1);

    expect(settled()).toBe(1);
  });

  it("gives a caller a promise for the target rather than for the camera", async () => {
    const { host, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    let isSettled: boolean = false;

    void scheduler.setTarget([0]).then(() => {
      isSettled = true;
    });

    await Promise.resolve();

    expect(isSettled).toBe(false);

    await arrive(0);

    expect(isSettled).toBe(true);
  });

  it("never shows a full bar, because the read that fills it is the one that ends the cycle", async () => {
    const { host, progress, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    void scheduler.setTarget([0, 1]);

    await arrive(0);
    await arrive(1);

    expect(progress).toEqual(["0/2", "1/2"]);
  });

  it("lets go of a target when the level does", async () => {
    const { host, settled, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    let isSettled: boolean = false;

    void scheduler.setTarget([0, 1]).then(() => {
      isSettled = true;
    });

    scheduler.clear();

    await Promise.resolve();

    expect(isSettled).toBe(true);

    // The read was already away and still arrives, but it belongs to a level nobody is holding, so nothing settles.
    await arrive(0);

    expect(settled()).toBe(0);
  });

  // The defect that broke opening a level. A sector number means nothing across levels: the read of the last
  // level's sector 0 was still in flight, so the new level's sector 0 was filtered out of its own queue and then
  // waited for a read the reader had already dropped. Nothing near the camera arrived, and flying somewhere with
  // different sector numbers was the only thing that looked like it worked.
  it("re-reads a sector whose last read belonged to a level that has gone", async () => {
    const { host, reads } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    void scheduler.setTarget([0, 1]);

    expect(asked(reads)).toEqual([0, 1]);

    scheduler.clear();

    void scheduler.setTarget([0, 1]);

    await Promise.resolve();

    expect(asked(reads)).toEqual([0, 1, 0, 1]);
  });

  // Nothing takes a sector back out of the in-flight set but the read finishing, so a host that throws where it
  // was expected to reject would wedge the queue for the rest of the level.
  it("survives a host that throws instead of rejecting", async () => {
    const { host, settled } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler({
      ...host,
      read: (): Promise<void> => {
        throw new Error("the session went away");
      },
    });

    await scheduler.setTarget([0, 1]);

    expect(settled()).toBe(1);
    expect(scheduler.reading).toBe(0);
  });

  it("does not re-enter its own pump when a read settles at once", async () => {
    const { host, settled } = mockHost();
    const started: Array<number> = [];
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler({
      ...host,
      read: (sector: number): Promise<void> => {
        started.push(sector);

        return Promise.resolve();
      },
    });

    scheduler.setConcurrency(2);

    await scheduler.setTarget([0, 1, 2, 3]);

    expect(started).toEqual([0, 1, 2, 3]);
    expect(settled()).toBe(1);
  });

  // Flying at the maximum boost crosses 2400 metres a second, which no read rate reaches. The only way to have a
  // sector when the camera arrives is to have read it before it asked.
  it("reads the rest of the level once the camera has what it wanted", async () => {
    const { host, reads, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    void scheduler.setTarget([0]);
    scheduler.setBackground([5, 6]);

    // Nothing yet: what the camera asked for is still in the air.
    expect(asked(reads)).toEqual([0]);

    await arrive(0);

    expect(asked(reads)).toEqual([0, 5]);

    await arrive(5);

    expect(asked(reads)).toEqual([0, 5, 6]);
  });

  // The fill must never be the reason a sector the camera is waiting on is queued behind something it is not.
  it("stands aside the moment the camera asks for something", async () => {
    const { host, reads, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    scheduler.setBackground([5, 6, 7]);

    expect(asked(reads)).toEqual([5]);

    void scheduler.setTarget([0, 1]);

    expect(asked(reads)).toEqual([5, 0, 1]);

    await arrive(0);
    await arrive(1);

    // Five was already away and six waits: what the camera wanted went first and settled without it.
    expect(asked(reads)).toEqual([5, 0, 1]);
  });

  it("does not count the fill as progress, nor settle on it", async () => {
    const { host, settled, progress, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    scheduler.setBackground([5]);

    await arrive(5);

    expect(settled()).toBe(0);
    expect(progress).toEqual([]);
  });

  it("stops filling when the level goes", async () => {
    const { host, reads, arrive } = mockHost();
    const scheduler: LevelStreamScheduler = new LevelStreamScheduler(host);

    scheduler.setBackground([5, 6]);
    scheduler.clear();

    await arrive(5);

    expect(asked(reads)).toEqual([5]);
  });
});
