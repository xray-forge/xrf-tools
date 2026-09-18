import { BufferGeometry } from "three";

import { createSectorGeometry } from "@/core/level/lib/level-sector-geometry";
import { ISectorViews } from "@/core/level/lib/level-sector-views";

/** One resident sector: what it holds, and what the renderer uploaded for it. */
export interface ILoadedSector {
  sector: number;
  views: ISectorViews;
  geometry: BufferGeometry;
}

/**
 * The sectors a level currently holds, and the GPU resources that came with them.
 */
export class LevelSectorSet {
  private readonly loaded: Map<number, ILoadedSector> = new Map();

  /**
   * @returns Sectors currently held, which is what a residency plan is computed against.
   */
  public keys(): Set<number> {
    return new Set(this.loaded.keys());
  }

  public has(sector: number): boolean {
    return this.loaded.has(sector);
  }

  public get size(): number {
    return this.loaded.size;
  }

  /**
   * Builds a sector's geometry and takes ownership of it.
   *
   * @param views - Views over the buffer the sector arrived in.
   * @returns The sector now held.
   */
  public adopt(views: ISectorViews): ILoadedSector {
    this.release(views.sector);

    const loaded: ILoadedSector = { geometry: createSectorGeometry(views), sector: views.sector, views };

    this.loaded.set(views.sector, loaded);

    return loaded;
  }

  /**
   * Releases one sector and the device memory it held.
   *
   * @param sector - Sector to release; releasing one that is not held does nothing.
   */
  public release(sector: number): void {
    this.loaded.get(sector)?.geometry.dispose();
    this.loaded.delete(sector);
  }

  /** Releases every sector, for teardown and for swapping levels. */
  public dispose(): void {
    for (const sector of this.loaded.keys()) {
      this.release(sector);
    }
  }

  /**
   * @returns A snapshot of what is held, for a view to render from.
   */
  public snapshot(): ReadonlyMap<number, ILoadedSector> {
    return new Map(this.loaded);
  }
}
