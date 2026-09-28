import { Maybe, Nullable } from "@xrf/types";

import { StaticRunPool } from "#/scene/static/static-run-pool";

/** A run a slot holds of one pool: where it starts, and how many. */
interface IHeldRun {
  start: number;
  count: number;
}

/**
 * What each slot holds of the run pools besides its record: its clusters, and a single draw's place or an instanced
 * draw's rows. A run is kept while the slot wants one as long, and taken again otherwise.
 */
export class StaticSlotHolds {
  private readonly holds: Map<number, Map<StaticRunPool, IHeldRun>> = new Map();
  private readonly allocate: (pool: StaticRunPool, count: number) => Nullable<number>;

  /**
   * @param allocate - Takes a run of a pool, grown where it has to be; null where it cannot.
   */
  public constructor(allocate: (pool: StaticRunPool, count: number) => Nullable<number>) {
    this.allocate = allocate;
  }

  /**
   * @param slot - A slot.
   * @param pool - A pool.
   * @param count - What the slot wants of it.
   * @returns Where its run starts, or null where there is no room and it holds none.
   */
  public hold(slot: number, pool: StaticRunPool, count: number): Nullable<number> {
    let runs: Maybe<Map<StaticRunPool, IHeldRun>> = this.holds.get(slot);

    if (!runs) {
      runs = new Map();
      this.holds.set(slot, runs);
    }

    const held: Maybe<IHeldRun> = runs.get(pool);

    if (held?.count === count) {
      return held.start;
    }

    this.drop(slot, pool);

    const start: Nullable<number> = this.allocate(pool, count);

    if (start !== null) {
      runs.set(pool, { count, start });
    }

    return start;
  }

  /**
   * @param slot - A slot holding nothing of a pool from now on.
   * @param pool - The pool.
   */
  public drop(slot: number, pool: StaticRunPool): void {
    const runs: Maybe<Map<StaticRunPool, IHeldRun>> = this.holds.get(slot);
    const held: Maybe<IHeldRun> = runs?.get(pool);

    if (runs && held) {
      pool.free(held.start, held.count);
      runs.delete(pool);
    }
  }

  /**
   * @param slot - A slot holding nothing of any pool from now on.
   */
  public release(slot: number): void {
    this.holds.get(slot)?.forEach((held: IHeldRun, pool: StaticRunPool) => pool.free(held.start, held.count));
    this.holds.delete(slot);
  }
}
