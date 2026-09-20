import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { levelsRawCommands } from "@/core/ipc/commands/levels-raw";
import { Session } from "@/core/ipc/session";
import { requireSessionId } from "@/core/ipc/session/session.utils";
import { LevelSource, SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import {
  createLevelResidency,
  DEFAULT_LEVEL_RESIDENCY,
  ILevelPoint,
  ILevelResidencyOptions,
  ILevelResidencyPlan,
  planLevelResidency,
} from "@/core/level/lib/level-residency";
import { ILoadedSector, LevelSectorSet } from "@/core/level/lib/level-sector-set";
import { listSectorTextures } from "@/core/level/lib/level-sector-textures";
import { createSectorViews, ISectorViews } from "@/core/level/lib/level-sector-views";
import { ILevelTextureLookup, LevelTextureSet } from "@/core/level/lib/level-texture-set";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Maybe, Nullable } from "@/lib/types/general";

/** Nothing in flight, which is also what a viewer sees before it has asked for anything. */
export const IDLE_LEVEL_STREAM: ILevelStreamProgress = { loaded: 0, total: 0 };

/** A level that is open: what it is, and where its sectors are. */
export interface IOpenLevel {
  selected: SessionSnapshot<SelectedLevelDescription>;
}

/** How far through the sectors a camera asked for the loader has got. */
export interface ILevelStreamProgress {
  /** Sectors the current plan asked for, or zero when nothing is streaming. */
  total: number;
  /** Sectors of it that have arrived. */
  loaded: number;
}

/**
 * Opens a compiled level and keeps the sectors near the camera resident.
 */
@Injectable()
export class LevelLoadService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(levelsCommands.closeLevel);

  /** The sectors held, and the geometry each owns. */
  private readonly held: LevelSectorSet = new LevelSectorSet();

  /** The level's uploaded textures, owned here and shared between the sectors that name them. */
  private readonly loaded: LevelTextureSet = new LevelTextureSet();

  /**
   * The read in flight for each sector, so a second ask joins it rather than starting another.
   */
  private readonly reading: Map<string, Promise<void>> = new Map();

  @Observable()
  public level: AsyncState<IOpenLevel> = AsyncState.idle();

  /** Resident sectors for a viewport to draw, replaced whole so a view re-renders on any change. */
  @Observable()
  public sectors: ReadonlyMap<number, ILoadedSector> = new Map();

  /** How much of the level is held at once, and how far out it is worth holding. */
  @Observable()
  public residency: ILevelResidencyOptions = DEFAULT_LEVEL_RESIDENCY;

  @Observable()
  public streaming: ILevelStreamProgress = IDLE_LEVEL_STREAM;

  /**
   * Whether the restore below has settled, one way or the other.
   */
  @Observable()
  public isReady: boolean = false;

  /**
   * @returns The level's textures, to read rather than to manage: their lifetime is this service's.
   */
  public get textures(): ILevelTextureLookup {
    return this.loaded;
  }

  /**
   * @returns Whether sectors are on their way.
   */
  @Computed()
  public get isStreaming(): boolean {
    return this.streaming.total > 0;
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Open a level and report what it is built out of.
   *
   * @param source - Level directory or mounted asset to open.
   * @param roots - Roots the level and its textures are searched in.
   */
  @LatestFlow("level")
  public *load(source: LevelSource, roots: XrayRoots): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Loading level:", source.kind === "directory" ? source.path : source.logicalPath);

    try {
      this.level = this.level.asLoading();

      const selected: SessionSnapshot<SelectedLevelDescription> = yield* call(
        this.session.open(levelsCommands.openLevel, source, roots)
      );

      this.releaseSectors();

      this.residency = createLevelResidency(selected.value.bounds?.boundingSphere.radius ?? 0);
      this.loaded.open(roots, selected.value.textures);
      this.level = this.level.asReady({ selected });

      this.log.info(
        "Level opened in:",
        formatDuration(timer.elapsed()),
        `${selected.value.sectors.length} sectors,`,
        `${selected.value.visuals} visuals`
      );
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Load error after:", formatDuration(timer.elapsed()), transformed);

      this.level = this.level.asFailed(transformed);
    }
  }

  /**
   * Takes back whatever level the backend still has open.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      await flowResult(this.restore());
    } catch (error: unknown) {
      this.log.error("Failed to restore the selected level:", transformError(error));
    } finally {
      runInAction(() => {
        this.isReady = true;
      });
    }
  }

  /**
   * Restores the native selection unless a user action has taken the level flow.
   */
  @ExclusiveFlow("level")
  public *restore(): TFlow {
    const snapshot = yield* call(levelsCommands.getLevel());

    this.session.adopt(snapshot);

    if (snapshot) {
      this.releaseSectors();
      this.level = this.level.asReady({ selected: snapshot });
    }
  }

  /**
   * Brings what the camera is near into residency, and releases what it has left.
   *
   * @param point - Where the camera is, in renderer space.
   */
  @LatestFlow("stream")
  public *stream(point: ILevelPoint): TFlow {
    const open: Nullable<IOpenLevel> = this.level.value;

    if (!open) {
      return;
    }

    const sessionId: string = requireSessionId(open.selected);
    const plan: ILevelResidencyPlan = planLevelResidency(
      open.selected.value.sectors,
      point,
      this.held.keys(),
      this.residency
    );

    if (!plan.load.length && !plan.evict.length) {
      return;
    }

    for (const sector of plan.evict) {
      this.held.release(sector);
    }

    runInAction(() => {
      this.streaming = { loaded: 0, total: plan.load.length };
    });

    // Nearest first, and published as each arrives: a viewer draws what is nearest while the rest is still reading,
    // rather than waiting for the whole plan.
    try {
      for (const sector of plan.load) {
        yield* call(this.readOnce(sessionId, sector, open.selected.value.surfaces));

        runInAction(() => {
          this.streaming = { loaded: this.streaming.loaded + 1, total: this.streaming.total };
        });
      }
    } finally {
      // Both however the flow ends: a cancelled move must not leave a viewer reporting a read that is not coming, nor
      // leave the textures of a sector that never arrived holding memory until the level closes.
      this.loaded.retain(this.listResidentTextures());

      runInAction(() => {
        this.streaming = IDLE_LEVEL_STREAM;
      });
    }

    this.publishSectors();
  }

  /**
   * Closes this loader's level without clearing a newer one.
   */
  @LatestFlow("level")
  public *close(): TFlow {
    yield* call(this.session.close());

    this.clearView();
  }

  /**
   * Abandons pending work and releases the level and every sector it held.
   */
  @BoundAction()
  public clear(): void {
    cancelFlow(this, "level");
    cancelFlow(this, "stream");
    this.session.release();

    this.clearView();
  }

  /**
   * Reads one sector, or joins the read already in flight for it.
   *
   * @param sessionId - The level opening the sector belongs to.
   * @param sector - Sector to read, by its index in the sectors chunk.
   * @param surfaces - The level's resolved shader table, for the sector to join its surfaces against.
   * @returns A promise that settles when the sector has been read, or when it has failed and said so.
   */
  private readOnce(sessionId: string, sector: number, surfaces: ReadonlyArray<XraySurfaceDescriptor>): Promise<void> {
    const key: string = `${sessionId}:${sector}`;
    const reading: Maybe<Promise<void>> = this.reading.get(key);

    if (reading) {
      return reading;
    }

    const started: Promise<void> = this.readSector(sessionId, sector, surfaces)
      .catch((error: unknown) => {
        this.log.error(`Failed to read sector ${sector}:`, transformError(error));
      })
      .finally(() => {
        this.reading.delete(key);
      });

    this.reading.set(key, started);

    return started;
  }

  /**
   * Packs one sector on the backend and takes ownership of what comes back.
   *
   * @param sessionId - The level opening the sector belongs to.
   * @param sector - Sector to read, by its index in the sectors chunk.
   * @param surfaces - The level's resolved shader table.
   */
  private async readSector(
    sessionId: string,
    sector: number,
    surfaces: ReadonlyArray<XraySurfaceDescriptor>
  ): Promise<void> {
    const timer: Timer = new Timer();

    const snapshot: SessionSnapshot<SectorDescription> = await levelsCommands.openSector(
      sessionId,
      crypto.randomUUID(),
      sector
    );

    const buffer: ArrayBuffer = await levelsRawCommands.readSector(sessionId, snapshot.sessionId);
    // Joined against the table the open resolved, so a surface arrives already knowing whether it is cut out.
    const views: ISectorViews = createSectorViews(snapshot.value, buffer, surfaces);

    // Before the sector is published, so a surface is never drawn untextured for a frame and then corrected.
    await this.loaded.load(listSectorTextures(views));

    // A read outlives the plan that asked for it, so it can outlive the level too: a sector of a level nobody has
    // open any more is dropped rather than adopted into whatever is open now.
    if (!this.isOpen(sessionId)) {
      this.log.info(`Dropping sector ${sector} of a level that is no longer open`);

      return;
    }

    this.held.adopt(views);
    this.publishSectors();

    this.log.info(
      "Sector read in:",
      formatDuration(timer.elapsed()),
      `sector ${sector},`,
      `${views.geometry.vertexCount} vertices,`,
      `${views.sections.length} draws,`,
      `${views.instances.length} instanced meshes`
    );
  }

  /**
   * @param sessionId - The level opening a read belongs to.
   * @returns Whether that level is still the one this loader holds.
   */
  private isOpen(sessionId: string): boolean {
    const open: Nullable<IOpenLevel> = this.level.value;

    return Boolean(open && open.selected.sessionId === sessionId);
  }

  /**
   * @returns Every texture reference the resident sectors name, which is what is worth keeping uploaded.
   */
  private listResidentTextures(): Set<string> {
    const references: Set<string> = new Set();

    for (const loaded of this.held.snapshot().values()) {
      for (const request of listSectorTextures(loaded.views)) {
        references.add(request.reference);
      }
    }

    return references;
  }

  private publishSectors(): void {
    runInAction(() => {
      this.sectors = this.held.snapshot();
    });
  }

  private clearView(): void {
    runInAction(() => {
      this.level = this.level.asIdle();
      this.streaming = IDLE_LEVEL_STREAM;
      this.releaseSectors();
    });
  }

  private releaseSectors(): void {
    this.held.dispose();
    this.loaded.dispose();
    this.sectors = new Map();
  }
}
