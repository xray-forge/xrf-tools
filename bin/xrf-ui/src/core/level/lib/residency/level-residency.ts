import { SectorOutline } from "@/core/ipc/types/xrf-visual";
import { DEFAULT_LEVEL_STREAM_CONCURRENCY } from "@/core/level/lib/stream/level-stream-scheduler";
import { Nullable } from "@/lib/types/general";

/** A point in renderer space, which is where the camera is asked from. */
export interface ILevelPoint {
  x: number;
  y: number;
  z: number;
}

/** How much of a level is held at once, and how far from the camera it is worth holding. */
export interface ILevelResidencyOptions {
  /**  Bytes of packed geometry held at once, which is what decides how much of a level survives a flight. */
  memoryBudget: number;
  /** Sectors held at once whatever the budget says, a guard against a bad estimate rather than a budget. */
  maxSectors: number;
  /** Nearest sectors held whatever the distances or the budget say. */
  minSectors: number;
  /** A sector nearer than this is loaded. */
  loadDistance: number;
  /** A sector already held is kept until it passes this, which has to be the larger of the two. */
  keepDistance: number;
  /** Reads in flight at once, so overlapping is a declared number rather than the shape of a loop. */
  concurrency: number;
  /** Whether idle time is spent reading the rest of the level, nearest first. */
  isPreloaded: boolean;
}

/** What to open, what to release, and what is held once both are done. */
export interface ILevelResidencyPlan {
  /** Sectors to open, nearest first, so the ones a camera is about to see arrive first. */
  load: Array<number>;
  /** Sectors to release, furthest first. */
  evict: Array<number>;
  /** Sectors held once the plan is applied, nearest first. */
  resident: Array<number>;
}

/** A sector ranked against the camera. */
interface IRankedSector {
  sector: number;
  distance: number;
  held: boolean;
}

/** What a sector not yet read is assumed to cost, until enough have been read to say better. */
export const ESTIMATED_SECTOR_BYTES: number = 1024 * 1024;

export const DEFAULT_LEVEL_RESIDENCY: ILevelResidencyOptions = {
  concurrency: DEFAULT_LEVEL_STREAM_CONCURRENCY,
  isPreloaded: true,
  keepDistance: 400,
  loadDistance: 250,
  // The editor may spend what the game spends, and this is geometry's share of it. Flying at the maximum boost
  // crosses 2400 metres a second, which no read rate reaches: the only way to have a sector when the camera
  // arrives is to have kept it from last time.
  memoryBudget: 768 * 1024 * 1024,
  maxSectors: 512,
  minSectors: 4,
};

const LOAD_FRACTION: number = 0.75;
const KEEP_FRACTION: number = 1.25;

/**
 * Residency scaled to the level it is for.
 *
 * @param radius - How far the level reaches, from the extent its sectors declare.
 * @returns Residency options for a level that size.
 */
export function createLevelResidency(radius: number): ILevelResidencyOptions {
  return {
    ...DEFAULT_LEVEL_RESIDENCY,
    keepDistance: Math.max(DEFAULT_LEVEL_RESIDENCY.keepDistance, radius * KEEP_FRACTION),
    loadDistance: Math.max(DEFAULT_LEVEL_RESIDENCY.loadDistance, radius * LOAD_FRACTION),
  };
}

/**
 * Distance from a point to a sector's enclosing sphere, which is zero for a camera inside it.
 *
 * @param outline - Sector to measure, whose declared sphere is the measure.
 * @param point - Where the camera is.
 * @returns Distance to the sphere's surface, or `null` when the sector declares no extent.
 */
export function getSectorDistance(outline: SectorOutline, point: ILevelPoint): Nullable<number> {
  if (!outline.bounds) {
    return null;
  }

  const { center, radius } = outline.bounds.boundingSphere;
  const x: number = (center.x ?? 0) - point.x;
  const y: number = (center.y ?? 0) - point.y;
  const z: number = (center.z ?? 0) - point.z;

  // A non-finite radius crosses the wire as null, which is a sector with no usable extent rather than one at zero.
  return Math.max(0, Math.sqrt(x * x + y * y + z * z) - (radius ?? 0));
}

/**
 * Decides which sectors a camera should be holding.
 *
 * @param outlines - What the level's sectors are and where, from `open_level`.
 * @param point - Where the camera is.
 * @param held - Sectors currently resident, against what each of them costs in bytes.
 * @param options - Budget and the two distances.
 * @returns What to load, what to evict, and what is resident afterwards.
 */
export function planLevelResidency(
  outlines: ReadonlyArray<SectorOutline>,
  point: ILevelPoint,
  held: ReadonlyMap<number, number>,
  options: ILevelResidencyOptions = DEFAULT_LEVEL_RESIDENCY
): ILevelResidencyPlan {
  const ranked: Array<IRankedSector> = [];

  let within: number = 0;

  for (const outline of outlines) {
    const distance: Nullable<number> = getSectorDistance(outline, point);

    // A sector reaching no drawable has nothing to draw, so it is never worth a read.
    if (distance === null) {
      continue;
    }

    const isHeld: boolean = held.has(outline.sector);
    const limit: number = isHeld ? options.keepDistance : options.loadDistance;

    ranked.push({ distance, held: isHeld, sector: outline.sector });

    if (distance <= limit) {
      within += 1;
    }
  }

  // Nearest first, and a sector already held wins a tie so the budget does not swap two equals every frame.
  ranked.sort((left: IRankedSector, right: IRankedSector) => {
    return left.distance - right.distance || Number(right.held) - Number(left.held);
  });

  // The distances decide how much is worth holding; the two counts decide how much is held regardless. A camera
  // outside the whole level still draws its nearest sectors rather than nothing.
  const take: number = Math.min(Math.max(within, options.minSectors), Math.max(0, options.maxSectors));
  const estimate: number = getSectorEstimate(held);
  const resident: Array<number> = [];

  let spent: number = 0;

  for (const it of ranked.slice(0, take)) {
    const cost: number = held.get(it.sector) ?? estimate;

    // The floor comes first: a budget too small for what the camera is standing in draws nothing at all, which is
    // worse than going over it.
    if (resident.length >= options.minSectors && spent + cost > options.memoryBudget) {
      break;
    }

    spent += cost;
    resident.push(it.sector);
  }

  const wanted: Set<number> = new Set(resident);

  return {
    evict: Array.from(held.keys()).filter((sector: number) => !wanted.has(sector)),
    load: resident.filter((sector: number) => !held.has(sector)),
    resident,
  };
}

/**
 * The order the rest of a level is worth reading in, once the camera has what it asked for.
 *
 * @param outlines - What the level's sectors are and where.
 * @param point - Where the camera is.
 * @param held - Sectors currently resident, which need no reading.
 * @returns Every other sector that declares an extent, nearest first.
 */
export function listLevelPreload(
  outlines: ReadonlyArray<SectorOutline>,
  point: ILevelPoint,
  held: ReadonlySet<number> | ReadonlyMap<number, number>
): Array<number> {
  const ranked: Array<IRankedSector> = [];

  for (const outline of outlines) {
    const distance: Nullable<number> = getSectorDistance(outline, point);

    if (distance !== null && !held.has(outline.sector)) {
      ranked.push({ distance, held: false, sector: outline.sector });
    }
  }

  return ranked.sort((left, right) => left.distance - right.distance).map((it) => it.sector);
}

/** What a sector is worth assuming to cost, from the ones already read. */
function getSectorEstimate(held: ReadonlyMap<number, number>): number {
  if (!held.size) {
    return ESTIMATED_SECTOR_BYTES;
  }

  let total: number = 0;

  for (const bytes of held.values()) {
    total += bytes;
  }

  return total / held.size;
}
