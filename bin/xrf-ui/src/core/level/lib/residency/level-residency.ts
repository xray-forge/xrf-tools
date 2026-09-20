import { SectorOutline } from "@/core/ipc/types/xrf-visual";
import { Nullable } from "@/lib/types/general";

/** A point in renderer space, which is where the camera is asked from. */
export interface ILevelPoint {
  x: number;
  y: number;
  z: number;
}

/** How much of a level is held at once, and how far from the camera it is worth holding. */
export interface ILevelResidencyOptions {
  /** Sectors held at once, whatever the distances say. */
  maxSectors: number;
  /** Nearest sectors held whatever the distances say. */
  minSectors: number;
  /** A sector nearer than this is loaded. */
  loadDistance: number;
  /** A sector already held is kept until it passes this, which has to be the larger of the two. */
  keepDistance: number;
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

export const DEFAULT_LEVEL_RESIDENCY: ILevelResidencyOptions = {
  keepDistance: 400,
  loadDistance: 250,
  maxSectors: 24,
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
 * @param held - Sectors currently resident.
 * @param options - Budget and the two distances.
 * @returns What to load, what to evict, and what is resident afterwards.
 */
export function planLevelResidency(
  outlines: ReadonlyArray<SectorOutline>,
  point: ILevelPoint,
  held: ReadonlySet<number>,
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
  const resident: Array<number> = ranked.slice(0, take).map((it) => it.sector);
  const wanted: Set<number> = new Set(resident);

  return {
    evict: Array.from(held).filter((sector: number) => !wanted.has(sector)),
    load: resident.filter((sector: number) => !held.has(sector)),
    resident,
  };
}
