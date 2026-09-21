import { Nullable } from "@/lib/types/general";

/** What the scheduler needs of whoever owns the level, so it owns none of it. */
export interface ILevelStreamSchedulerHost {
  /** Reads one sector, settling when it is resident or has failed and said so. Never rejects. */
  read(sector: number): Promise<void>;
  /** Whether a sector is already held, since a target names what should be resident, not what is missing. */
  isResident(sector: number): boolean;
  /** How far through the target the reads have got, for a viewer to report. */
  report(loaded: number, total: number): void;
  /** Every wanted sector is resident or has failed, which is the only moment retention is worth doing. */
  settle(): void;
}

/** Reads in flight at once. Three overlaps a pack with the transfer and the adoption of the two before it. */
export const DEFAULT_LEVEL_STREAM_CONCURRENCY: number = 3;

/** A waiter for the target the scheduler is working towards. */
interface ISettlement {
  promise: Promise<void>;
  resolve: () => void;
}

/**
 * Brings a target set of sectors into residency.
 */
export class LevelStreamScheduler {
  private readonly host: ILevelStreamSchedulerHost;

  private concurrency: number = DEFAULT_LEVEL_STREAM_CONCURRENCY;

  /** Sectors that should be resident, which is what decides whether a read is still worth finishing. */
  private wanted: Set<number> = new Set();

  /** What is left to start, nearest first. */
  private queue: Array<number> = [];

  private readonly inFlight: Set<number> = new Set();

  /** Whether a target is being worked towards, so a report that changes nothing settles nothing. */
  private isRunning: boolean = false;

  private loaded: number = 0;
  private total: number = 0;

  private settlement: Nullable<ISettlement> = null;

  public constructor(host: ILevelStreamSchedulerHost) {
    this.host = host;
  }

  /**
   * @returns Sectors being read right now.
   */
  public get reading(): number {
    return this.inFlight.size;
  }

  /**
   * @param concurrency - Reads to keep in flight, at least one.
   */
  public setConcurrency(concurrency: number): void {
    this.concurrency = Math.max(1, concurrency);

    if (this.isRunning) {
      this.pump();
    }
  }

  /**
   * Takes the sectors that should be resident.
   *
   * @param sectors - What to hold, nearest first, whether or not it is already held.
   * @returns Settles once every one of them is resident or has failed, which is what a caller waits on rather than
   *   what a camera does.
   */
  public setTarget(sectors: ReadonlyArray<number>): Promise<void> {
    this.wanted = new Set(sectors);
    this.queue = sectors.filter((sector: number) => !this.inFlight.has(sector) && !this.host.isResident(sector));

    if (this.isSatisfied()) {
      // A camera that has moved without changing what is wanted costs one plan and nothing else. Settling here
      // when nothing was running is what used to run the level's housekeeping on every frame of a flight.
      if (this.isRunning) {
        this.finish();
      }

      return Promise.resolve();
    }

    this.isRunning = true;
    this.total = this.loaded + this.queue.length + this.countWantedInFlight();

    const settling: Promise<void> = this.settle();

    this.host.report(this.loaded, this.total);
    this.pump();

    return settling;
  }

  /**
   * Abandons the target, for a level closing or another opening.
   */
  public clear(): void {
    const settlement: Nullable<ISettlement> = this.settlement;

    this.wanted = new Set();
    this.queue = [];
    this.inFlight.clear();
    this.isRunning = false;
    this.loaded = 0;
    this.total = 0;
    this.settlement = null;

    settlement?.resolve();
  }

  /** Starts reads until the target is drained or the concurrency is reached. */
  private pump(): void {
    while (this.inFlight.size < this.concurrency && this.queue.length) {
      const sector: number = this.queue.shift() as number;

      // Both can have become true since the target was taken: a sector arrives, or a second target queues one that
      // the first already started.
      if (this.inFlight.has(sector) || this.host.isResident(sector)) {
        continue;
      }

      this.inFlight.add(sector);

      // Nothing takes a sector out of the in-flight set but its read finishing, so a host that throws where it
      // was expected to reject would wedge the queue for the rest of the level.
      let reading: Promise<void>;

      try {
        reading = this.host.read(sector);
      } catch {
        reading = Promise.resolve();
      }

      void reading.then(
        () => this.onRead(sector),
        () => this.onRead(sector)
      );
    }
  }

  private onRead(sector: number): void {
    this.inFlight.delete(sector);

    // A read outlives the target that asked for it, and can outlive the level: the cycle it belonged to is over,
    // so there is nothing left to count it towards and nothing to settle on its account.
    if (!this.isRunning) {
      return;
    }

    // A sector the camera left while it was packing still arrived, and the plan that dropped it will evict it. It
    // is not progress towards what is wanted now, so it is not counted as any.
    if (this.wanted.has(sector)) {
      this.loaded += 1;
    }

    this.pump();

    if (this.isSatisfied()) {
      this.finish();
    } else {
      this.host.report(this.loaded, this.total);
    }
  }

  /**
   * @returns Whether everything wanted is resident or has failed. A read of a sector nobody wants any more is not
   *   waited for: it cannot be called back, and nothing is waiting on what it brings.
   */
  private isSatisfied(): boolean {
    return !this.queue.length && !this.countWantedInFlight();
  }

  private countWantedInFlight(): number {
    let counted: number = 0;

    for (const sector of this.inFlight) {
      if (this.wanted.has(sector)) {
        counted += 1;
      }
    }

    return counted;
  }

  /** Ends the cycle: the counters reset and the host does its housekeeping, in that order and exactly once. */
  private finish(): void {
    const settlement: Nullable<ISettlement> = this.settlement;

    this.isRunning = false;
    this.loaded = 0;
    this.total = 0;
    this.settlement = null;

    // Before the waiters, and with the counters already reset: a viewer never shows a full bar, because the read
    // that filled it is the same one that ended the cycle.
    this.host.settle();

    settlement?.resolve();
  }

  private settle(): Promise<void> {
    if (!this.settlement) {
      // Taken out of the executor, which runs before the constructor returns, so the handle is in hand by the time
      // anything can settle it.
      let resolve: Nullable<() => void> = null;
      const promise: Promise<void> = new Promise<void>((it: () => void) => {
        resolve = it;
      });

      this.settlement = { promise, resolve: resolve as unknown as () => void };
    }

    return this.settlement.promise;
  }
}
