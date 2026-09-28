import { atomicAdd, atomicLoad, atomicStore, Break, Fn, If, instanceIndex, int, Return, uint } from "three/tsl";
import { ComputeNode, Node } from "three/webgpu";

import { IGrassCacheBuffers } from "#/scene/grass/grass-cache-buffers";
import { toGrassBandCounts, toGrassCacheKeys, toGrassSchedule } from "#/scene/grass/grass-cache.tsl";
import { IGrassRingSlot } from "#/scene/grass/grass-ring-slot";
import { isGrassCellCurrent, toGrassRingCells, toGrassRingSlot } from "#/scene/grass/grass-ring.tsl";
import { loopNamed } from "#/shader/named-loop.tsl";
import { GrassRingUniforms } from "#/uniforms/grass-ring-uniforms";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

/**
 * Slots a frame plants at most: `dm_max_decompress`, the engine's own seven on the CPU, grown to a GPU that plants
 * them side by side. A step of the camera stales a row of the ring, well within it; a jump stales the whole ring,
 * which then fills over some frames, the nearest first.
 */
export const GRASS_DECOMPRESS_BUDGET: number = 2048;

/** Where each word of the schedule stands. */
const STOP_WORD: number = 0;
const LEFT_WORD: number = 1;
const TICKET_WORD: number = 2;

/**
 * @param cache - The ring.
 * @returns The pass zeroing the stale cells each band counts.
 */
export function createGrassScheduleClear(cache: IGrassCacheBuffers): ComputeNode {
  const counts = toGrassBandCounts(cache);

  return Fn(() => {
    atomicStore(counts.element(instanceIndex), uint(0));
  })().compute(cache.bands);
}

/**
 * `cache_Update`'s task list: every stale cell of the ring, counted into the band it stands in.
 *
 * @param cache - The ring.
 * @param uniforms - Where the camera stands and what the planting runs under.
 * @param ring - How far the ring reaches.
 * @returns The pass, a thread a cell; its dispatch is the ring's cells, set before each frame's.
 */
export function createGrassRank(
  cache: IGrassCacheBuffers,
  uniforms: GrassUniforms,
  ring: GrassRingUniforms
): ComputeNode {
  const keys = toGrassCacheKeys(cache).toReadOnly();
  const counts = toGrassBandCounts(cache);

  return Fn(() => {
    const cell = int(instanceIndex).toVar();

    If(cell.greaterThanEqual(toGrassRingCells(ring)), () => {
      Return();
    });

    const slot: IGrassRingSlot = toGrassRingSlot(uniforms, ring, cell);

    If(isGrassCellCurrent(keys, cell, slot, uint(uniforms.generation)), () => {
      Return();
    });

    atomicAdd(counts.element(uint(slot.band)), uint(1));
  })().compute(cache.cells);
}

/**
 * The task performer's pick, nearest first: the band the frame's budget runs out in, and how many of its cells it
 * still plants there.
 *
 * @param cache - The ring.
 * @returns The pass, one thread.
 */
export function createGrassSelect(cache: IGrassCacheBuffers): ComputeNode {
  const counts = toGrassBandCounts(cache);
  const schedule = toGrassSchedule(cache);

  return Fn(() => {
    const taken = uint(0).toVar();
    // Past every band: all that is stale is planted.
    const stop = uint(cache.bands).toVar();
    const left = uint(0).toVar();

    loopNamed({ end: uint(cache.bands), name: "band", start: uint(0), type: "uint" }, (band: Node<"uint">) => {
      const count = (atomicLoad(counts.element(band)) as unknown as Node<"uint">).toVar();

      If(taken.add(count).greaterThan(uint(GRASS_DECOMPRESS_BUDGET)), () => {
        stop.assign(band);
        left.assign(uint(GRASS_DECOMPRESS_BUDGET).sub(taken));
        Break();
      });

      taken.addAssign(count);
    });

    atomicStore(schedule.element(STOP_WORD), stop);
    atomicStore(schedule.element(LEFT_WORD), left);
    atomicStore(schedule.element(TICKET_WORD), uint(0));
  })().compute(1);
}

/**
 * Whether a stale cell is planted this frame: any in a band nearer than the one the budget runs out in, and in that
 * one the first to take a ticket, as many as the budget has left. Takes a ticket where it has to ask, so is read once.
 *
 * @param cache - The ring.
 * @param band - The cell's band.
 * @returns Whether it is planted.
 */
export function isGrassCellAdmitted(cache: IGrassCacheBuffers, band: Node<"int">): Node<"bool"> {
  const schedule = toGrassSchedule(cache);
  const stop = (atomicLoad(schedule.element(STOP_WORD)) as unknown as Node<"uint">).toVar();
  const at = uint(band).toVar();
  const isAdmitted = at.lessThan(stop).toVar();

  If(at.equal(stop), () => {
    const ticket = atomicAdd(schedule.element(TICKET_WORD), uint(1)) as unknown as Node<"uint">;

    isAdmitted.assign(ticket.lessThan(atomicLoad(schedule.element(LEFT_WORD)) as unknown as Node<"uint">));
  });

  return isAdmitted;
}
