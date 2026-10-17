import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, flowResult, Observable, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { formatDuration } from "@xrf/format";
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
import { describeLevelSource } from "@/core/level/lib/source/level-source";
import { EMPTY_LEVEL_SPAWN_REPORT, ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
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

  /** The last open asked for, which a retry asks for again. */
  private request: Nullable<LevelOpenRequest> = null;

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  @Observable()
  public level: AsyncState<IOpenLevel> = AsyncState.idle();

  /** The level being opened, named before it is open: what a viewer heads its viewport with while it waits. */
  @RefObservable()
  public opening: Nullable<LevelSource> = null;

  /** Opens begun, so one taken over by a later open leaves that one's name. */
  private openings: number = 0;

  /** The objects the open level's spawn places, once listed; null before, and for a spawn that could not be read. */
  @RefObservable()
  public spawn: Nullable<LevelSpawnObjectsDescription> = null;

  /** How the spawn's listing went. */
  @RefObservable()
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
   * Open a level and report what it is built out of. One that fails closes the level open before it, so the viewer
   * shows the failure rather than the level it was asked to replace.
   *
   * @param request - The level to open, the roots it is searched in, and how the game's configs beside it are read.
   */
  @LatestFlow("level")
  public *load(request: LevelOpenRequest): TFlow {
    const { source } = request;
    const timer: Timer = new Timer();
    const opening: number = ++this.openings;

    this.request = request;
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

      if (this.level.value) {
        yield* call(this.release());
      }

      this.level = this.level.asFailed(transformed, null);
      this.spawn = null;
      this.spawnReport = EMPTY_LEVEL_SPAWN_REPORT;
    } finally {
      // A later open took the flow over meanwhile, and names its own level.
      if (opening === this.openings) {
        this.opening = null;
      }
    }
  }

  /**
   * Opens the last level asked for again, after it failed.
   */
  public async retry(): Promise<void> {
    if (this.request) {
      await flowResult(this.load(this.request));
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
   * Closes this loader's level without clearing a newer one. The view is cleared even where the backend failed to let
   * the level go, which stays owned for the next close to release.
   */
  @LatestFlow("level")
  public *close(): TFlow {
    yield* call(this.release());

    this.clearView();
  }

  /**
   * Lets the backend's level go; a failure is told as a notice, the level staying owned for the next close.
   */
  private async release(): Promise<void> {
    const source: Maybe<LevelSource> = this.level.value?.selected.value.source;

    try {
      await this.session.close();

      if (source) {
        this.log.info("Level closed:", describeLevelSource(source));
      }
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to close level:", transformed);

      emitNotification(this.eventBus, {
        details: source ? `${describeLevelSource(source)}\n${transformed.message}` : transformed.message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.LEVEL_VIEWER,
        title: "Could not close the level",
      });
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
   * Tells a listener the level open now, then again each time another opens or it closes, until stopped.
   *
   * @param listener - Hears the open level, or null for none.
   * @returns What stops it.
   */
  public followLevel(listener: (selected: Nullable<SessionSnapshot<SelectedLevelDescription>>) => void): () => void {
    return reaction(() => this.level.value?.selected ?? null, listener, { fireImmediately: true });
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
