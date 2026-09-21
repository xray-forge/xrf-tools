import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { Session } from "@/core/ipc/session";
import { requireSessionId } from "@/core/ipc/session/session.utils";
import { LevelSource, SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import {
  createLevelResidency,
  DEFAULT_LEVEL_RESIDENCY,
  ILevelPoint,
  ILevelResidencyOptions,
  ILevelResidencyPlan,
  planLevelResidency,
} from "@/core/level/lib/residency/level-residency";
import { LevelSectorReader } from "@/core/level/lib/sector/level-sector-reader";
import { ILoadedSector, LevelSectorSet } from "@/core/level/lib/sector/level-sector-set";
import { ISectorTextureRequest, listSectorTextures } from "@/core/level/lib/sector/level-sector-textures";
import { ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import {
  EMPTY_LEVEL_STREAM_SUMMARY,
  ILevelStreamReading,
  ILevelStreamSummary,
  LevelStreamProfile,
} from "@/core/level/lib/stream/level-stream-profile";
import { ILevelTextureLookup, LevelTextureSet } from "@/core/level/lib/texture/level-texture-set";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

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

  /** What the reads have cost, kept here because the loader is what owns a read from end to end. */
  private readonly profile: LevelStreamProfile = new LevelStreamProfile();

  /**
   * Reads sectors on this loader's behalf, joining a read already in flight and holding its textures until the
   * sector it read is resident or dropped.
   */
  private readonly reader: LevelSectorReader = new LevelSectorReader({
    adopt: (views: ISectorViews): void => {
      this.held.adopt(views);
      this.publishSectors();
    },
    isOpen: (sessionId: string): boolean => this.isOpen(sessionId),
    load: async (requests: ReadonlyArray<ISectorTextureRequest>): Promise<void> => {
      await this.loaded.load(requests);

      this.noteTextures();
    },
    record: (reading: ILevelStreamReading): void => this.noteReading(reading),
  });

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

  @Observable()
  public textureRevision: number = 0;

  /** What the recent sector reads cost, stage by stage, for a viewer to report and a change to be judged against. */
  @Observable()
  public streamProfile: ILevelStreamSummary = EMPTY_LEVEL_STREAM_SUMMARY;

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

      this.adopt(selected);

      this.log.info(
        "Level opened in:",
        formatDuration(timer.elapsed()),
        source.kind === "directory" ? source.path : source.logicalPath,
        `${selected.value.sectors.length} sectors,`,
        `${selected.value.visuals} visuals`
      );
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to load level:",
        source.kind === "directory" ? source.path : source.logicalPath,
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

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
      this.adopt(snapshot);

      const source: LevelSource = snapshot.value.source;

      this.log.info(
        "Level restored:",
        source.kind === "directory" ? source.path : source.logicalPath,
        snapshot.sessionId
      );
    }
  }

  /**
   * Takes one opening as the level this loader holds.
   *
   * @param selected - The opening the backend holds, from an open or from a restore.
   */
  @BoundAction()
  private adopt(selected: SessionSnapshot<SelectedLevelDescription>): void {
    this.releaseSectors();

    this.residency = createLevelResidency(selected.value.bounds?.boundingSphere.radius ?? 0);
    // The roots the **open** searched, not the ones a caller happens to hold: they are centred on the level, which is
    // what finds a texture shipped beside it, and a restore has no other way to know them.
    this.loaded.open(selected.value.roots, selected.value.textures);
    this.level = this.level.asReady({ selected });

    // The open released whatever the last level held, so everything drawn from the set is now undressed.
    this.textureRevision += 1;
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
        yield* call(this.reader.read(sessionId, sector, open.selected.value.surfaces));

        runInAction(() => {
          this.streaming = { loaded: this.streaming.loaded + 1, total: this.streaming.total };
        });
      }
    } finally {
      // Both however the flow ends: a cancelled move must not leave a viewer reporting a read that is not coming, nor
      // leave the textures of a sector that never arrived holding memory until the level closes.
      this.loaded.retain(this.listResidentTextures());
      this.noteTextures();

      runInAction(() => {
        this.streaming = IDLE_LEVEL_STREAM;
      });
    }

    this.publishSectors();
  }

  /** Records that the texture set is not what anything drawing from it last saw. */
  @BoundAction()
  private noteTextures(): void {
    this.textureRevision += 1;
  }

  /**
   * Takes what one sector cost and republishes the summary.
   *
   * @param reading - What that sector's read came to, stage by stage.
   */
  @BoundAction()
  private noteReading(reading: ILevelStreamReading): void {
    this.profile.record(reading);
    this.streamProfile = this.profile.summarise();
  }

  /**
   * Closes this loader's level without clearing a newer one.
   */
  @LatestFlow("level")
  public *close(): TFlow {
    const source = this.level.value?.selected.value.source;

    yield* call(this.session.close());

    this.clearView();

    if (source) {
      this.log.info("Level closed:", source.kind === "directory" ? source.path : source.logicalPath);
    }
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
   * @param sessionId - The level opening a read belongs to.
   * @returns Whether that level is still the one this loader holds.
   */
  private isOpen(sessionId: string): boolean {
    const open: Nullable<IOpenLevel> = this.level.value;

    return Boolean(open && open.selected.sessionId === sessionId);
  }

  /**
   * @returns Every texture reference a resident sector names or a read in flight has claimed, which is what is worth
   *   keeping uploaded.
   */
  private listResidentTextures(): Set<string> {
    const references: Set<string> = new Set();

    for (const loaded of this.held.snapshot().values()) {
      for (const request of listSectorTextures(loaded.views)) {
        references.add(request.reference);
      }
    }

    for (const reference of this.reader.listClaimedTextures()) {
      references.add(reference);
    }

    return references;
  }

  private publishSectors(): void {
    runInAction(() => {
      this.sectors = this.held.snapshot();
    });
  }

  private clearView(): void {
    this.profile.clear();
    this.streamProfile = EMPTY_LEVEL_STREAM_SUMMARY;

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
