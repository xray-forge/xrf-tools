import { BufferGeometry } from "three";

import { ILevelHeld } from "@/core/level/lib/stats/level-stats";

import { createSectorGeometry } from "./level-sector-geometry";
import { ISectorViews } from "./level-sector-views";

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

  /** What each held sector costs, kept in step with the sectors themselves. */
  private readonly bytes: Map<number, number> = new Map();

  /**
   * @returns Sectors currently held, which is what a residency plan is computed against.
   */
  public keys(): Set<number> {
    return new Set(this.loaded.keys());
  }

  /**
   * What each held sector costs, in bytes of the buffer it arrived in.
   *
   * @returns The sizes, which the caller must not modify.
   */
  public sizes(): ReadonlyMap<number, number> {
    return this.bytes;
  }

  /**
   * @returns How much is held, which is what a viewer reports and a budget is spent against.
   */
  public measure(): ILevelHeld {
    let bytes: number = 0;

    for (const held of this.bytes.values()) {
      bytes += held;
    }

    return { bytes, sectors: this.loaded.size };
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
    this.bytes.set(views.sector, views.bufferLength);

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
    this.bytes.delete(sector);
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
