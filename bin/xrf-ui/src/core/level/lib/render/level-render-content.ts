import { toMean } from "@xrf/math";
import { IDdsRead, IDdsRefusal, readDdsFile, RendererClient } from "@xrf/renderer";
import { Maybe } from "@xrf/types";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { SectorSurface } from "@/core/ipc/types/xrf-visual";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import {
  ILevelSectorChange,
  ILevelSectorDelivery,
  ILevelTextureDelivery,
  ILevelTextureSupplyChange,
} from "@/core/level/lib/render/level-render-protocol";
import {
  toLevelInstanceGeometry,
  toLevelInstanceObject,
  toLevelSectorGeometry,
  toLevelSectorObject,
} from "@/core/level/lib/render/level-render-sector";
import { toLevelSurface } from "@/core/level/lib/render/level-render-surface";
import { createLevelCheckerSource, toLevelTextureSource } from "@/core/level/lib/render/level-render-texture";
import { createSectorViews, ISectorInstanceViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { ILevelHeld } from "@/core/level/lib/stats/level-stats";
import { countSectorSurfaceGeometry, mergeLevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-count";
import { ELevelSurfaceDressing, ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "@/core/level/lib/surface/level-surface-options";
import { ILevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";
import { ILevelTextureProblem, ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { Timer } from "@/lib/logging";

/** What of the renderer a level is put through. */
export type TLevelRenderSink = Pick<
  RendererClient,
  | "putGeometry"
  | "releaseGeometry"
  | "putObject"
  | "releaseObject"
  | "putSurface"
  | "releaseSurface"
  | "putTexture"
  | "releaseTexture"
>;

/** Arrivals the mean is taken over, which matches the read profile's window so the two numbers are comparable. */
const ADD_WINDOW: number = 32;

/** One sector the renderer holds, and what it came to. */
interface IHeldSector {
  geometries: Array<string>;
  objects: Array<string>;
  bytes: number;
  /** What each entry draws in it, counted as it arrived: its bytes went to the renderer. */
  geometry: ReadonlyMap<number, ILevelSurfaceGeometry>;
}

/** A shader table entry put, with what its row names, so a toggle can put it again. */
interface IPutSurface {
  surface: SectorSurface;
  render: ILevelSurfaceRender;
}

/**
 * The open level as the renderer holds it: sectors and textures put as they arrive and released as they go, one
 * surface per shader table entry, and what each of them came to for the panels.
 */
export class LevelRenderContent {
  private readonly sink: TLevelRenderSink;
  private table: ReadonlyArray<XraySurfaceDescriptor> = [];
  private readonly surfaces: Map<number, IPutSurface> = new Map();
  private readonly sectors: Map<number, IHeldSector> = new Map();
  /** What became of each reference supplied, in the order they were. */
  private readonly textures: Map<string, ILevelSurfaceDressing> = new Map();
  private options: ILevelSurfaceOptions = DEFAULT_LEVEL_SURFACE_OPTIONS;
  /** What the recent arrivals cost to put, which is the half of a sector's arrival no read stage covers. */
  private readonly added: Array<number> = [];

  public constructor(sink: TLevelRenderSink) {
    this.sink = sink;
  }

  /**
   * Takes the level whose shader table every arriving sector joins against, letting the last one go.
   *
   * @param table - The level's resolved shader table, empty for no level.
   */
  public open(table: ReadonlyArray<XraySurfaceDescriptor>): void {
    this.clear();
    this.table = table;
  }

  /**
   * @param change - Sectors that arrived and sectors that went.
   */
  public deliver(change: ILevelSectorChange): void {
    for (const sector of change.released ?? Array.from(this.sectors.keys())) {
      this.drop(sector);
    }

    change.delivered.forEach((delivery: ILevelSectorDelivery) => this.take(delivery));
  }

  /**
   * @param change - Files that were read, and what is still worth keeping.
   */
  public supply(change: ILevelTextureSupplyChange): void {
    for (const delivery of change.delivered) {
      this.sink.putTexture(delivery.reference, toLevelTextureSource(delivery));
      this.textures.set(delivery.reference, LevelRenderContent.toDressing(delivery));
    }

    if (change.retained !== null) {
      for (const reference of Array.from(this.textures.keys())) {
        if (!change.retained.has(reference)) {
          this.releaseTexture(reference);
        }
      }
    } else if (!change.delivered.length) {
      Array.from(this.textures.keys()).forEach((reference: string) => this.releaseTexture(reference));
    }
  }

  /**
   * Stands a checker in for a file the renderer could not upload as stored.
   *
   * @param reference - What it was put under.
   * @param refusal - Why.
   */
  public refuse(reference: string, refusal: IDdsRefusal): void {
    if (!this.textures.has(reference)) {
      return;
    }

    this.sink.putTexture(reference, createLevelCheckerSource());
    this.textures.set(reference, {
      reason: `The renderer would not upload it: ${refusal.detail}`,
      reference,
      state: ELevelSurfaceDressing.STOOD_IN,
      upload: null,
    });
  }

  /**
   * @param options - How the toolbar has the surfaces drawn; every entry is put again when that changes them.
   */
  public setOptions(options: ILevelSurfaceOptions): void {
    const isChanged: boolean = options.isTextured !== this.options.isTextured;

    this.options = options;

    if (isChanged) {
      this.surfaces.forEach(({ surface, render }, shaderId: number) =>
        this.sink.putSurface(LEVEL_RENDER_KEYS.surface(shaderId), toLevelSurface(surface, render, options))
      );
    }
  }

  /**
   * @returns What each shader table entry draws across the sectors held.
   */
  public measure(): ReadonlyMap<number, ILevelSurfaceGeometry> {
    return mergeLevelSurfaceGeometry(Array.from(this.sectors.values(), (it: IHeldSector) => it.geometry));
  }

  /**
   * @returns How much is held.
   */
  public held(): ILevelHeld {
    let bytes: number = 0;

    this.sectors.forEach((it: IHeldSector) => (bytes += it.bytes));

    return { bytes, sectors: this.sectors.size };
  }

  /**
   * @returns Mean milliseconds one arriving sector has been costing to put, or zero before any has.
   */
  public get meanAddTime(): number {
    return toMean(this.added);
  }

  /**
   * @returns What the level's textures came to.
   */
  public describeTextures(): ILevelTextureReport {
    const problems: Array<ILevelTextureProblem> = [];

    this.textures.forEach((dressing: ILevelSurfaceDressing, reference: string) => {
      if (dressing.reason) {
        problems.push({ reason: dressing.reason, reference });
      }
    });

    return { dressing: new Map(this.textures), problems, uploaded: this.textures.size };
  }

  /** Lets everything the level put go. */
  public clear(): void {
    Array.from(this.sectors.keys()).forEach((sector: number) => this.drop(sector));
    this.surfaces.forEach((_, shaderId: number) => this.sink.releaseSurface(LEVEL_RENDER_KEYS.surface(shaderId)));
    this.surfaces.clear();
    Array.from(this.textures.keys()).forEach((reference: string) => this.releaseTexture(reference));
    this.table = [];
    this.added.length = 0;
  }

  private take(delivery: ILevelSectorDelivery): void {
    const timer: Timer = new Timer();

    this.drop(delivery.sector);

    const views: ISectorViews = createSectorViews(delivery.description, delivery.buffer, this.table);
    const held: IHeldSector = {
      bytes: views.bufferLength,
      geometries: [],
      // Counted now: the views are over bytes that move to the renderer once this task ends.
      geometry: countSectorSurfaceGeometry(views),
      objects: [],
    };

    // Only where the level bakes something in place. A sector whose drawables it all places has an empty index
    // array, and an object drawing none of it would be a draw call to say nothing.
    if (views.sections.length) {
      const key: string = LEVEL_RENDER_KEYS.sector(views.sector);

      views.sections.forEach(({ surface, render }) => this.ensureSurface(surface, render));
      this.sink.putGeometry(key, toLevelSectorGeometry(views));
      this.sink.putObject(key, toLevelSectorObject(views));
      held.geometries.push(key);
      held.objects.push(key);
    }

    views.instances.forEach((instance: ISectorInstanceViews, index: number) => {
      const key: string = LEVEL_RENDER_KEYS.instance(views.sector, index);

      this.ensureSurface(instance.surface, instance.render);
      this.sink.putGeometry(key, toLevelInstanceGeometry(instance));
      this.sink.putObject(key, toLevelInstanceObject(views.sector, index, instance));
      held.geometries.push(key);
      held.objects.push(key);
    });

    this.sectors.set(views.sector, held);
    this.note(timer.elapsed());
  }

  private drop(sector: number): void {
    const held: Maybe<IHeldSector> = this.sectors.get(sector);

    if (held) {
      held.objects.forEach((key: string) => this.sink.releaseObject(key));
      held.geometries.forEach((key: string) => this.sink.releaseGeometry(key));
      this.sectors.delete(sector);
    }
  }

  /** Puts a shader table entry the first time a sector draws it: every sector drawing it shares it after. */
  private ensureSurface(surface: SectorSurface, render: ILevelSurfaceRender): void {
    if (!this.surfaces.has(surface.shaderId)) {
      this.surfaces.set(surface.shaderId, { render, surface });
      this.sink.putSurface(LEVEL_RENDER_KEYS.surface(surface.shaderId), toLevelSurface(surface, render, this.options));
    }
  }

  private releaseTexture(reference: string): void {
    this.sink.releaseTexture(reference);
    this.textures.delete(reference);
  }

  /** Keeps what the recent arrivals cost, over the same window the read profile means its stages over. */
  private note(elapsed: number): void {
    this.added.push(elapsed);

    if (this.added.length > ADD_WINDOW) {
      this.added.shift();
    }
  }

  /** What one delivered file came to, as a panel reads it: described here, before its bytes move to the renderer. */
  private static toDressing(delivery: ILevelTextureDelivery): ILevelSurfaceDressing {
    const { reference } = delivery;

    if (delivery.reason) {
      return { reason: delivery.reason, reference, state: ELevelSurfaceDressing.STOOD_IN, upload: null };
    }

    if (delivery.isDecoded) {
      return { reason: null, reference, state: ELevelSurfaceDressing.UPLOADED, upload: "decoded by the backend" };
    }

    const read: IDdsRead = readDdsFile(delivery.bytes);
    const levels: number = read.file?.mipmaps.length ?? 0;

    return {
      reason: null,
      reference,
      state: ELevelSurfaceDressing.UPLOADED,
      upload: read.file
        ? `${read.file.width}×${read.file.height} · ${levels} ${levels === 1 ? "level" : "levels"}`
        : null,
    };
  }
}
