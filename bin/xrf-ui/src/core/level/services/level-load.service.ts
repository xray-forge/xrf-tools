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
  listLevelPreload,
  planLevelResidency,
} from "@/core/level/lib/residency/level-residency";
import { ILevelSectorAdoption, LevelSectorReader } from "@/core/level/lib/sector/level-sector-reader";
import { ILoadedSector, LevelSectorSet } from "@/core/level/lib/sector/level-sector-set";
import { ISectorTextureRequest, listSectorTextures } from "@/core/level/lib/sector/level-sector-textures";
import { ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import {
  EMPTY_LEVEL_STREAM_SUMMARY,
  ILevelStreamReading,
  ILevelStreamSummary,
  LevelStreamProfile,
} from "@/core/level/lib/stream/level-stream-profile";
import { LevelStreamScheduler } from "@/core/level/lib/stream/level-stream-scheduler";
import { ILevelTextureSource, LevelTextureSet } from "@/core/level/lib/texture/level-texture-set";
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

  /** Where the camera last reported from, which is what the level is filled in around once it settles. */
  private streamedFrom: Nullable<ILevelPoint> = null;

  /**
   * Reads sectors on this loader's behalf, joining a read already in flight and holding its textures until the
   * sector it read is resident or dropped.
   */
  private readonly reader: LevelSectorReader = new LevelSectorReader({
    adopt: (views: ISectorViews): ILevelSectorAdoption => {
      const stage: Timer = new Timer();

      this.held.adopt(views);

      const geometry: number = stage.lap();

      this.publishSectors();

      return { geometry, publish: stage.lap() };
    },
    isOpen: (sessionId: string): boolean => this.isOpen(sessionId),
    load: (requests: ReadonlyArray<ISectorTextureRequest>): Promise<void> => this.loaded.load(requests),
    record: (reading: ILevelStreamReading): void => this.noteReading(reading),
  });

  /**
   * Brings what the camera wants into residency.
   *
   * A camera report sets its target and returns; it starts nothing itself and cancels nothing, which is what keeps
   * a flight from spending every frame abandoning the read it asked for on the frame before.
   */
  private readonly scheduler: LevelStreamScheduler = new LevelStreamScheduler({
    isResident: (sector: number): boolean => this.held.has(sector),
    read: (sector: number): Promise<void> => this.readSector(sector),
    report: (loaded: number, total: number): void => this.noteProgress(loaded, total),
    settle: (): void => this.onSettled(),
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

  /** What the recent sector reads cost, stage by stage, for a viewer to report and a change to be judged against. */
  @Observable()
  public streamProfile: ILevelStreamSummary = EMPTY_LEVEL_STREAM_SUMMARY;

  /**
   * Whether the restore below has settled, one way or the other.
   */
  @Observable()
  public isReady: boolean = false;

  /**
   * @returns The level's textures, to read and to hear about rather than to manage: their lifetime is this
   *   service's, and the set says for itself what changed.
   */
  public get textures(): ILevelTextureSource {
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

    // Before anything else: a target belongs to the level it was planned against, and so does every sector number
    // in it. The level about to open inherits neither.
    this.scheduler.clear();

    this.residency = createLevelResidency(selected.value.bounds?.boundingSphere.radius ?? 0);
    this.scheduler.setConcurrency(this.residency.concurrency);
    // The roots the **open** searched, not the ones a caller happens to hold: they are centred on the level, which is
    // what finds a texture shipped beside it, and a restore has no other way to know them. Releases the last level's
    // textures itself, and says so once rather than once for the release and again for the open.
    this.loaded.open(selected.value.roots, selected.value.textures);
    this.level = this.level.asReady({ selected });
  }

  /**
   * Brings what the camera is near into residency, and releases what it has left.
   *
   * @param point - Where the camera is, in renderer space.
   */
  public stream(point: ILevelPoint): Promise<void> {
    const open: Nullable<IOpenLevel> = this.level.value;

    if (!open) {
      return Promise.resolve();
    }

    const timer: Timer = new Timer();

    this.streamedFrom = point;

    const plan: ILevelResidencyPlan = planLevelResidency(
      open.selected.value.sectors,
      point,
      this.held.sizes(),
      this.residency
    );

    if (plan.evict.length) {
      for (const sector of plan.evict) {
        this.held.release(sector);
      }

      this.publishSectors();
    }

    // The whole target, not the part of it that is missing: what is already held is what tells the scheduler which
    // of its reads are still worth finishing.
    const settled: Promise<void> = this.scheduler.setTarget(plan.resident);

    // Measured here rather than around the plan alone, because what a camera report costs the frame it lands on is
    // everything this method does - and answering one used to include abandoning whatever was in flight.
    this.noteReport(timer.elapsed());

    return settled;
  }

  /**
   * Reads one sector of the level this loader holds.
   *
   * @param sector - Sector to read, by its index in the sectors chunk.
   * @returns Settles when it is resident or has failed and said so.
   */
  private readSector(sector: number): Promise<void> {
    const open: Nullable<IOpenLevel> = this.level.value;

    return open
      ? this.reader.read(requireSessionId(open.selected), sector, open.selected.value.surfaces)
      : Promise.resolve();
  }

  /**
   * Everything the camera wants is resident or has failed.
   *
   * Retention happens here and nowhere else. It reads every resident sector's surfaces, which is far too much to
   * pay on a camera report - and under boost a report lands every frame.
   */
  @BoundAction()
  private onSettled(): void {
    this.loaded.retain(this.listResidentTextures());
    this.streaming = IDLE_LEVEL_STREAM;

    // Only now, and only from here: it walks every sector of the level, which is far too much for a camera
    // report, and there is nothing to fill in with until the camera has what it asked for.
    if (this.residency.isPreloaded) {
      const open: Nullable<IOpenLevel> = this.level.value;

      this.scheduler.setBackground(
        open && this.streamedFrom
          ? listLevelPreload(open.selected.value.sectors, this.streamedFrom, this.held.keys())
          : []
      );
    }
  }

  @BoundAction()
  private noteProgress(loaded: number, total: number): void {
    this.streaming = { loaded, total };
  }

  /**
   * Takes what one camera report cost and republishes the summary.
   *
   * @param elapsed - Milliseconds it took on the thread that draws.
   */
  @BoundAction()
  private noteReport(elapsed: number): void {
    this.profile.recordReport(elapsed);
    this.streamProfile = this.profile.summarise();
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
    this.scheduler.clear();
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
    this.scheduler.clear();
    this.profile.clear();
    this.streamedFrom = null;
    this.streamProfile = EMPTY_LEVEL_STREAM_SUMMARY;

    runInAction(() => {
      this.level = this.level.asIdle();
      this.streaming = IDLE_LEVEL_STREAM;
      this.releaseSectors();
      this.loaded.dispose();
    });
  }

  private releaseSectors(): void {
    this.held.dispose();
    this.sectors = new Map();
  }
}
