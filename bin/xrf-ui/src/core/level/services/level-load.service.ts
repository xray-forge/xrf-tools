import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, flowResult, Observable, runInAction } from "@wirestate/mobx";
import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { Session } from "@/core/ipc/session";
import {
  LevelOpenRequest,
  LevelSource,
  LevelSpawnObjectsDescription,
  SelectedLevelDescription,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { describeLevelSource } from "@/core/level/lib/source";
import { EMPTY_LEVEL_SPAWN_REPORT, ILevelSpawnReport } from "@/core/level/lib/spawn";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

/** A level that is open: what it is. */
export interface IOpenLevel {
  selected: SessionSnapshot<SelectedLevelDescription>;
}

/**
 * Opens a compiled level, and lists its spawned objects for the panels: the renderer reads and draws everything else
 * the level holds itself.
 */
@Injectable()
export class LevelLoadService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(levelsCommands.closeLevel);

  @Observable()
  public level: AsyncState<IOpenLevel> = AsyncState.idle();

  /** The level being opened, named before it is open: what a viewer heads its viewport with while it waits. */
  @Observable()
  public opening: Nullable<LevelSource> = null;

  /** Opens begun, so one taken over by a later open leaves that one's name. */
  private openings: number = 0;

  /** The objects the open level's spawn places, once listed; null before, and for a spawn that could not be read. */
  @Observable()
  public spawn: Nullable<LevelSpawnObjectsDescription> = null;

  /** How the spawn's listing went. */
  @Observable()
  public spawnReport: ILevelSpawnReport = EMPTY_LEVEL_SPAWN_REPORT;

  /**
   * Whether the restore below has settled, one way or the other.
   */
  @Observable()
  public isReady: boolean = false;

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Open a level and report what it is built out of.
   *
   * @param request - The level to open, the roots it is searched in, and how the game's configs beside it are read.
   */
  @LatestFlow("level")
  public *load(request: LevelOpenRequest): TFlow {
    const { source } = request;
    const timer: Timer = new Timer();
    const opening: number = ++this.openings;

    this.log.info("Loading level:", describeLevelSource(source));

    try {
      this.opening = source;
      this.level = this.level.asLoading();

      const selected: SessionSnapshot<SelectedLevelDescription> = yield* call(
        this.session.open(levelsCommands.openLevel, request)
      );

      this.adopt(selected);

      this.log.info(
        "Level opened in:",
        formatDuration(timer.elapsed()),
        describeLevelSource(source),
        `${selected.value.sectors.length} sectors,`,
        `${selected.value.visuals} visuals`
      );
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to load level:",
        describeLevelSource(source),
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.level = this.level.asFailed(transformed);
    } finally {
      // A later open took the flow over meanwhile, and names its own level.
      if (opening === this.openings) {
        this.opening = null;
      }
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
    const snapshot: SessionRestore<SelectedLevelDescription> = yield* call(levelsCommands.getLevel());

    this.session.adopt(snapshot);

    if (snapshot) {
      this.adopt(snapshot);
      this.log.info("Level restored:", describeLevelSource(snapshot.value.source), snapshot.sessionId);
    }
  }

  /**
   * Takes one opening as the level this loader holds, and lists its spawned objects.
   *
   * @param selected - The opening the backend holds, from an open or from a restore.
   */
  @BoundAction()
  private adopt(selected: SessionSnapshot<SelectedLevelDescription>): void {
    this.level = this.level.asReady({ selected });
    this.spawn = null;
    this.spawnReport = EMPTY_LEVEL_SPAWN_REPORT;
    void this.readSpawn(selected.sessionId);
  }

  /** The level's spawned objects, which the panels list and a pick names. */
  private async readSpawn(sessionId: string): Promise<void> {
    const timer: Timer = new Timer();

    try {
      const { value }: SessionSnapshot<LevelSpawnObjectsDescription> = await levelsCommands.openSpawnObjects(sessionId);

      if (this.isOpen(sessionId)) {
        this.log.info(
          "Level spawned objects listed in:",
          formatDuration(timer.elapsed()),
          `${value.objects.length} objects,`,
          `${value.visuals.length} visuals`
        );
        this.noteSpawn(value, {
          failure: null,
          isListed: true,
          objects: value.objects.length,
          visuals: value.visuals.length,
        });
      }
    } catch (error: unknown) {
      this.log.error("Failed to read the level's spawned objects:", transformError(error));

      if (this.isOpen(sessionId)) {
        this.noteSpawn(null, { ...EMPTY_LEVEL_SPAWN_REPORT, failure: transformError(error).message, isListed: true });
      }
    }
  }

  @BoundAction()
  private noteSpawn(spawn: Nullable<LevelSpawnObjectsDescription>, report: ILevelSpawnReport): void {
    this.spawn = spawn;
    this.spawnReport = report;
  }

  /**
   * Closes this loader's level without clearing a newer one.
   */
  @LatestFlow("level")
  public *close(): TFlow {
    const source: Maybe<LevelSource> = this.level.value?.selected.value.source;

    yield* call(this.session.close());

    this.clearView();

    if (source) {
      this.log.info("Level closed:", describeLevelSource(source));
    }
  }

  /**
   * Abandons pending work and releases the level.
   */
  @BoundAction()
  public clear(): void {
    cancelFlow(this, "level");
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

  private clearView(): void {
    runInAction(() => {
      this.level = this.level.asIdle();
      this.spawn = null;
      this.spawnReport = EMPTY_LEVEL_SPAWN_REPORT;
    });
  }
}
