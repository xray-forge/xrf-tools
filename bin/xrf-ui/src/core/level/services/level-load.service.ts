import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Observable, runInAction } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { levelsRawCommands } from "@/core/ipc/commands/levels-raw";
import { Session } from "@/core/ipc/session";
import { requireSessionId } from "@/core/ipc/session/session.utils";
import { LevelSource, SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import {
  DEFAULT_LEVEL_RESIDENCY,
  ILevelPoint,
  ILevelResidencyOptions,
  ILevelResidencyPlan,
  planLevelResidency,
} from "@/core/level/lib/level-residency";
import { ILoadedSector, LevelSectorSet } from "@/core/level/lib/level-sector-set";
import { createSectorViews, ISectorViews } from "@/core/level/lib/level-sector-views";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** A level that is open: what it is, and where its sectors are. */
export interface IOpenLevel {
  selected: SessionSnapshot<SelectedLevelDescription>;
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

  @Observable()
  public level: AsyncState<IOpenLevel> = AsyncState.idle();

  /** Resident sectors for a viewport to draw, replaced whole so a view re-renders on any change. */
  @Observable()
  public sectors: ReadonlyMap<number, ILoadedSector> = new Map();

  /** How much of the level is held at once, and how far out it is worth holding. */
  @Observable()
  public residency: ILevelResidencyOptions = DEFAULT_LEVEL_RESIDENCY;

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

    // Nearest first, and published as each arrives: a viewer draws what is nearest while the rest is still reading,
    // rather than waiting for the whole plan.
    for (const sector of plan.load) {
      yield* this.readSector(sessionId, sector);
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
   * Packs one sector on the backend and takes ownership of what comes back.
   *
   * @param sessionId - The level opening the sector belongs to.
   * @param sector - Sector to read, by its index in the sectors chunk.
   */
  private *readSector(sessionId: string, sector: number): TFlow {
    const timer: Timer = new Timer();

    const snapshot: SessionSnapshot<SectorDescription> = yield* call(
      levelsCommands.openSector(sessionId, crypto.randomUUID(), sector)
    );

    const buffer: ArrayBuffer = yield* call(levelsRawCommands.readSector(sessionId, snapshot.sessionId));
    const views: ISectorViews = createSectorViews(snapshot.value, buffer);

    this.held.adopt(views);
    this.publishSectors();

    this.log.info(
      "Sector read in:",
      formatDuration(timer.elapsed()),
      `sector ${sector},`,
      `${views.vertexCount} vertices,`,
      `${views.sections.length} draws`
    );
  }

  private publishSectors(): void {
    runInAction(() => {
      this.sectors = this.held.snapshot();
    });
  }

  private clearView(): void {
    runInAction(() => {
      this.level = this.level.asIdle();
      this.releaseSectors();
    });
  }

  private releaseSectors(): void {
    this.held.dispose();
    this.sectors = new Map();
  }
}
