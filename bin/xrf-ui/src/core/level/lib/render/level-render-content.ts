import { toMean } from "@xrf/math";
import { IRendererTextureFetch, RendererClient } from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import { LevelLightsDescription } from "@/core/ipc/types/xrf-app";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { SectorSurface } from "@/core/ipc/types/xrf-visual";
import { toLevelRendererGrass } from "@/core/level/lib/render/level-render-grass";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import { toLevelRendererLights } from "@/core/level/lib/render/level-render-lights";
import {
  ILevelGrassDelivery,
  ILevelSectorChange,
  ILevelSectorDelivery,
  ILevelTextureDelivery,
  ILevelTextureSupplyChange,
} from "@/core/level/lib/render/level-render-protocol";
import {
  createLevelImpostorQuad,
  toLevelImpostorObject,
  toLevelImpostors,
  toLevelInstanceGeometry,
  toLevelInstanceObject,
  toLevelSectorGeometry,
  toLevelSectorObject,
} from "@/core/level/lib/render/level-render-sector";
import {
  ILevelSpawnModelsDelivery,
  ILevelSpawnPart,
  toLevelSpawnParts,
  toLevelSpawnSurface,
} from "@/core/level/lib/render/level-render-spawn";
import { toLevelSurface } from "@/core/level/lib/render/level-render-surface";
import { createLevelCheckerSource, toLevelTextureSource } from "@/core/level/lib/render/level-render-texture";
import { createSectorViews, ISectorInstanceViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { ILevelHeld } from "@/core/level/lib/stats/level-stats";
import { countSectorSurfaceGeometry, mergeLevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-count";
import { ELevelSurfaceDressing, ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
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
  | "putImpostors"
  | "releaseImpostors"
  | "putGrass"
  | "releaseGrass"
  | "putLights"
  | "releaseLights"
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
  /** The impostor set it put, or null for a sector with no clump of trees. */
  impostors: Nullable<string>;
  bytes: number;
  /** What each entry draws in it, counted as it arrived: its bytes went to the renderer. */
  geometry: ReadonlyMap<number, ILevelSurfaceGeometry>;
}

/**
 * The open level as the renderer holds it: sectors and textures put as they arrive and released as they go, one
 * surface per shader table entry, and what each of them came to for the panels.
 */
export class LevelRenderContent {
  private readonly sink: TLevelRenderSink;
  private table: ReadonlyArray<XraySurfaceDescriptor> = [];
  /** The shader table entries put, by id. */
  private readonly surfaces: Set<number> = new Set();
  private readonly sectors: Map<number, IHeldSector> = new Map();
  /** What became of each reference supplied, in the order they were. */
  private readonly textures: Map<string, ILevelSurfaceDressing> = new Map();
  /** The spawned models' parts put. */
  private spawnParts: ReadonlyArray<ILevelSpawnPart> = [];
  /** Whether the quad every impostor draws over is put. */
  private isImpostorQuadPut: boolean = false;
  /** What the recent arrivals cost to put, which is the half of a sector's arrival no read stage covers. */
  private readonly added: Array<number> = [];

  public constructor(sink: TLevelRenderSink) {
    this.sink = sink;
  }

  /**
   * Takes the level whose shader table every arriving sector joins against, letting the last one's sectors, surfaces
   * and textures go. Its grass, lights and spawned models are the loader's to let go, which it does as it opens.
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
   * @param grass - The level's grass, or null for none.
   */
  public plant(grass: Nullable<ILevelGrassDelivery>): void {
    if (grass) {
      this.sink.putGrass(toLevelRendererGrass(grass));
    } else {
      this.sink.releaseGrass();
    }
  }

  /**
   * @param lights - The level's lights, or null for none.
   */
  public light(lights: Nullable<LevelLightsDescription>): void {
    if (lights) {
      this.sink.putLights(toLevelRendererLights(lights));
    } else {
      this.sink.releaseLights();
    }
  }

  /**
   * @param models - The models the level's spawned objects stand as, or null for none; every part put before goes.
   */
  public stand(models: Nullable<ILevelSpawnModelsDelivery>): void {
    this.releaseSpawnParts();
    this.spawnParts = models ? toLevelSpawnParts(models) : [];

    for (const part of this.spawnParts) {
      this.sink.putGeometry(part.key, part.geometry);
      this.sink.putSurface(part.key, toLevelSpawnSurface(part.dressing));
      this.sink.putObject(part.key, part.object);
    }
  }

  /**
   * @param change - Files to fetch, and what is still worth keeping.
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
   * Takes what a file the renderer fetched came to, standing a checker in for one it could not fetch.
   *
   * @param reference - What it was put under.
   * @param fetch - What it came to.
   */
  public fetched(reference: string, fetch: IRendererTextureFetch): void {
    // Released since, or put again as a checker: what it came to is nobody's any more.
    if (this.textures.get(reference)?.state !== ELevelSurfaceDressing.FETCHING) {
      return;
    }

    if (fetch.failure) {
      this.sink.putTexture(reference, createLevelCheckerSource());
      this.textures.set(reference, {
        reason: fetch.failure,
        reference,
        state: ELevelSurfaceDressing.STOOD_IN,
        upload: null,
      });

      return;
    }

    this.textures.set(reference, {
      reason: null,
      reference,
      state: ELevelSurfaceDressing.UPLOADED,
      upload: LevelRenderContent.describeUpload(fetch),
    });
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
    let uploaded: number = 0;

    this.textures.forEach((dressing: ILevelSurfaceDressing, reference: string) => {
      if (dressing.reason) {
        problems.push({ reason: dressing.reason, reference });
      }

      // A checker is uploaded as much as a file is; a fetch on its way is neither yet.
      if (dressing.state !== ELevelSurfaceDressing.FETCHING) {
        uploaded += 1;
      }
    });

    return { dressing: new Map(this.textures), problems, uploaded };
  }

  /** Lets the level's sectors, surfaces and textures go. */
  private clear(): void {
    Array.from(this.sectors.keys()).forEach((sector: number) => this.drop(sector));
    this.surfaces.forEach((shaderId: number) => this.sink.releaseSurface(LEVEL_RENDER_KEYS.surface(shaderId)));
    this.surfaces.clear();

    if (this.isImpostorQuadPut) {
      this.sink.releaseGeometry(LEVEL_RENDER_KEYS.impostorQuad);
      this.isImpostorQuadPut = false;
    }

    Array.from(this.textures.keys()).forEach((reference: string) => this.releaseTexture(reference));
    this.table = [];
    this.added.length = 0;
  }

  private releaseSpawnParts(): void {
    for (const { key } of this.spawnParts) {
      this.sink.releaseObject(key);
      this.sink.releaseSurface(key);
      this.sink.releaseGeometry(key);
    }

    this.spawnParts = [];
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
      impostors: null,
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

    if (views.impostors) {
      this.takeImpostors(views.sector, views.impostors, held);
    }

    this.sectors.set(views.sector, held);
    this.note(timer.elapsed());
  }

  private drop(sector: number): void {
    const held: Maybe<IHeldSector> = this.sectors.get(sector);

    if (held) {
      held.objects.forEach((key: string) => this.sink.releaseObject(key));
      held.geometries.forEach((key: string) => this.sink.releaseGeometry(key));

      if (held.impostors) {
        this.sink.releaseImpostors(held.impostors);
      }

      this.sectors.delete(sector);
    }
  }

  /** Puts a sector's impostor set, before the objects naming it, and a draw for each run of it by surface. */
  private takeImpostors(sector: number, impostors: NonNullable<ISectorViews["impostors"]>, held: IHeldSector): void {
    const key: string = LEVEL_RENDER_KEYS.impostors(sector);

    if (!this.isImpostorQuadPut) {
      this.sink.putGeometry(LEVEL_RENDER_KEYS.impostorQuad, createLevelImpostorQuad());
      this.isImpostorQuadPut = true;
    }

    this.sink.putImpostors(key, toLevelImpostors(impostors));
    held.impostors = key;

    impostors.groups.forEach(({ surface, render }, group: number) => {
      const object: string = LEVEL_RENDER_KEYS.impostorGroup(sector, group);

      this.ensureSurface(surface, render);
      this.sink.putObject(object, toLevelImpostorObject(sector, impostors, group));
      held.objects.push(object);
    });
  }

  /** Puts a shader table entry the first time a sector draws it: every sector drawing it shares it after. */
  private ensureSurface(surface: SectorSurface, render: ILevelSurfaceRender): void {
    if (!this.surfaces.has(surface.shaderId)) {
      this.surfaces.add(surface.shaderId);
      this.sink.putSurface(LEVEL_RENDER_KEYS.surface(surface.shaderId), toLevelSurface(surface, render));
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

  /** What one delivered file is until the renderer says what it came to: on its way, or a checker from the start. */
  private static toDressing(delivery: ILevelTextureDelivery): ILevelSurfaceDressing {
    const { reference } = delivery;

    return delivery.requests
      ? { reason: null, reference, state: ELevelSurfaceDressing.FETCHING, upload: null }
      : { reason: delivery.reason, reference, state: ELevelSurfaceDressing.STOOD_IN, upload: null };
  }

  /** How a fetched file was uploaded, as a panel reads it. */
  private static describeUpload(fetch: IRendererTextureFetch): Nullable<string> {
    const { size } = fetch;

    if (fetch.isDecoded) {
      return "decoded by the backend";
    }

    return size ? `${size.width}×${size.height} · ${size.levels} ${size.levels === 1 ? "level" : "levels"}` : null;
  }
}
