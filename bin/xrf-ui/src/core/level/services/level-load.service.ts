import { EventBus, inject, Injectable, OnDeactivation, OnEvent, OnProvision, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { formatDuration } from "@xrf/format";
import { Logger, Timer } from "@xrf/logging";
import { assertExhaustive, Maybe, Nullable } from "@xrf/types";

import { levelsCommands } from "@/core/ipc/commands/levels";
import {
  isOlderSessionAnswer,
  isSessionMovedPast,
  Session,
  SESSION_CHANGE_EVENT,
  SessionPartResource,
  SessionScopeService,
} from "@/core/ipc/session";
import { ESessionRootChange, SessionChange, SessionId, SessionSnapshot } from "@/core/ipc/types/xrf-app-core";
import {
  LEVEL_MAP_PART,
  LEVEL_SPAWN_PART,
  LEVEL_START_PART,
  LEVEL_SURFACES_PART,
  LEVEL_TEXTURES_PART,
  LevelMapDescription,
  LevelOpenRequest,
  LevelSource,
  LevelSpawnObjectsDescription,
  LevelStart,
  LevelTextureReference,
  SelectedLevelDescription,
} from "@/core/ipc/types/xrf-app-levels";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { XrayRevision } from "@/core/ipc/types/xrf-vfs";
import { describeLevelSource } from "@/core/level/lib/source/level-source";
import { EMPTY_LEVEL_SPAWN_REPORT, ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

/** A level that is open: the level itself, its root, as its session holds it. */
export interface IOpenLevel {
  selected: SessionSnapshot<SelectedLevelDescription>;
}

/** The scene an open level streams as: its session, and the revision it last streamed afresh at. */
export interface ILevelScene {
  sessionId: SessionId;
  revision: XrayRevision;
}

/**
 * Opens a compiled level, the level itself, and keeps each part of it the page shows - where it starts, its map, its
 * surfaces and their textures, its spawned objects - as a resource read apart from it and again apart from it: the
 * renderer reads and draws everything else the level holds itself.
 */
@Injectable()
export class LevelLoadService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(levelsCommands.closeLevel, inject(SessionScopeService));

  /** The last open asked for, which a retry asks for again. */
  private request: Nullable<LevelOpenRequest> = null;

  /** Where the open level opens, or none for a viewer to decide. */
  public readonly start: SessionPartResource<Nullable<LevelStart>>;

  /** Where the open level's map lies over it, or none where its config places none. */
  public readonly levelMap: SessionPartResource<Nullable<LevelMapDescription>>;

  /** How the renderer draws each entry of the open level's shader table, indexed by the shader id a surface carries. */
  public readonly surfaces: SessionPartResource<Array<XraySurfaceDescriptor>>;

  /** What every texture the open level's surfaces bind resolved to. */
  public readonly textures: SessionPartResource<Array<LevelTextureReference>>;

  /** The objects the open level's spawn places, which the panels list and a pick names. */
  public readonly spawnObjects: SessionPartResource<LevelSpawnObjectsDescription>;

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {
    this.start = new SessionPartResource(eventBus, [LEVEL_START_PART], levelsCommands.describeStart);
    this.levelMap = new SessionPartResource(eventBus, [LEVEL_MAP_PART], levelsCommands.describeLevelMap);
    this.surfaces = new SessionPartResource(eventBus, [LEVEL_SURFACES_PART], levelsCommands.describeSurfaces);
    this.textures = new SessionPartResource(eventBus, [LEVEL_TEXTURES_PART], levelsCommands.describeTextures);
    this.spawnObjects = new SessionPartResource(eventBus, [LEVEL_SPAWN_PART], levelsCommands.describeSpawnObjects);
  }

  @Observable()
  public level: AsyncState<IOpenLevel> = AsyncState.idle();

  /** The level being opened, named before it is open: what a viewer heads its viewport with while it waits. */
  @RefObservable()
  public opening: Nullable<LevelSource> = null;

  /** Opens begun, so one taken over by a later open leaves that one's name. */
  private openings: number = 0;

  /**
   * The changes heard while an open or a restore is under way, for any opening but the level shown: the new opening's
   * id is known only once its answer resumes the flow, and its start is read before it is adopted, so a change to it
   * may come first. Those of the opening adopted, not older than it, are heard then.
   */
  private heard: Nullable<Array<SessionChange>> = null;

  /**
   * The scene the open level streams as, moved on each time the backend streams it afresh: the level derived anew, or
   * what its sectors are packed and drawn from changed. None while no level is open.
   */
  @RefObservable()
  public scene: Nullable<ILevelScene> = null;

  /** Whether a change moved what the open level was read from: it shows as it opened until it is read again. */
  @Observable()
  public isStale: boolean = false;

  /**
   * Whether the restore below has settled, one way or the other.
   */
  @Observable()
  public isReady: boolean = false;

  /** The objects the open level's spawn places, once listed; null before, and for a spawn that could not be read. */
  @Computed()
  public get spawn(): Nullable<LevelSpawnObjectsDescription> {
    return this.spawnObjects.error ? null : this.spawnObjects.value;
  }

  /** How the spawn's listing went. */
  @Computed()
  public get spawnReport(): ILevelSpawnReport {
    const { error, value } = this.spawnObjects;

    if (error) {
      return { ...EMPTY_LEVEL_SPAWN_REPORT, failure: error.message, isListed: true };
    }

    return value
      ? { failure: null, isListed: true, objects: value.objects.length, visuals: value.visuals.length }
      : EMPTY_LEVEL_SPAWN_REPORT;
  }

  /** The revision the spawn's listing was read at; null before one is. */
  @Computed()
  public get spawnRevision(): Nullable<XrayRevision> {
    return this.spawnObjects.snapshot?.revision ?? null;
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Open a level and report what it is built out of: where it starts read before it shows, the rest of its parts
   * after. One that fails closes the level open before it, so the viewer shows the failure rather than the level it was
   * asked to replace.
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
      this.heard = [];
      this.level = this.level.asLoading();

      const selected: SessionSnapshot<SelectedLevelDescription> = yield* call(
        this.session.open(levelsCommands.openLevel, request)
      );

      yield* call(this.start.attach(selected.sessionId));

      this.adopt(selected);

      yield* call(this.attachParts(selected.sessionId));

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

      this.heard = null;
      this.detachParts();
      this.level = this.level.asFailed(transformed, null);
      this.scene = null;
      this.isStale = false;
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
   * Restores the native selection unless a user action has taken the level flow: where it starts read before it shows,
   * the rest of its parts after.
   */
  @ExclusiveFlow("level")
  public *restore(): TFlow {
    this.heard = [];

    const snapshot: Nullable<SessionSnapshot<SelectedLevelDescription>> = yield* call(
      levelsCommands.restoreLevel(this.session.tool)
    );

    this.session.adopt(snapshot);

    if (!snapshot) {
      this.heard = null;
    } else {
      yield* call(this.start.attach(snapshot.sessionId));

      this.adopt(snapshot);
      this.log.info("Level restored:", describeLevelSource(snapshot.value.source), snapshot.sessionId);

      yield* call(this.attachParts(snapshot.sessionId));
    }
  }

  /**
   * Takes one opening as the level this loader holds, the scene it streams as with it, letting the parts of the level
   * held before go at once, so nothing shows one level's parts beside another; its start is read already. The changes
   * heard for it while it was opened are heard now, those not older than the snapshot adopted: a change that derives,
   * leaves stale or fails it comes at its own revision.
   *
   * @param selected - The opening the backend holds, from an open or a restore.
   */
  @BoundAction()
  private adopt(selected: SessionSnapshot<SelectedLevelDescription>): void {
    const heard: Array<SessionChange> = (this.heard ?? []).filter(
      (change: SessionChange) => change.sessionId === selected.sessionId && !isOlderSessionAnswer(selected, change)
    );

    this.heard = null;
    this.level = this.level.asReady({ selected });
    this.scene = { revision: selected.revision, sessionId: selected.sessionId };
    this.isStale = false;
    this.levelMap.detach();
    this.surfaces.detach();
    this.textures.detach();
    this.spawnObjects.detach();

    for (const change of heard) {
      this.follow(selected, change);
    }
  }

  /**
   * Reads the level itself again once its session derived it anew, and moves its scene on once the backend streams it
   * afresh; a level whose own files moved is marked stale meanwhile, and left as it opened where it could not be read
   * again. Each part reads itself again as a change names it. A change heard while an open or a restore is under way
   * waits for its adopt.
   */
  @OnEvent(SESSION_CHANGE_EVENT)
  public onSessionChange({ payload: change }: WireEvent<SessionChange>): void {
    if (!change) {
      return;
    }

    const open: Maybe<SessionSnapshot<SelectedLevelDescription>> = this.level.value?.selected;

    if (open && open.sessionId === change.sessionId) {
      this.follow(open, change);
    } else {
      this.heard?.push(change);
    }
  }

  /** Follows one change to the level shown, `open`. */
  private follow(open: SessionSnapshot<SelectedLevelDescription>, change: SessionChange): void {
    this.noteScene(change);

    switch (change.root.kind) {
      case ESessionRootChange.KEPT:
        return;

      case ESessionRootChange.DERIVED:
        if (isSessionMovedPast(open, change)) {
          this.noteStale();
          void this.refresh();
        }

        return;

      case ESessionRootChange.DERIVING:
      case ESessionRootChange.STALE:
      case ESessionRootChange.FAILED:
        this.noteStale();

        return;

      default:
        return assertExhaustive(change.root);
    }
  }

  /** Moves the scene on for a change past the one it streams as that streams it afresh, as the backend says. */
  @BoundAction()
  private noteScene(change: SessionChange): void {
    if (
      this.scene &&
      change.isViewRenewed &&
      isSessionMovedPast({ revision: this.scene.revision, sessionId: this.scene.sessionId }, change)
    ) {
      this.scene = { revision: change.revision, sessionId: change.sessionId };
    }
  }

  @BoundAction()
  private noteStale(): void {
    this.isStale = true;
  }

  /** Reads the committed level again, for `onSessionChange`; a failure leaves what is shown, stale. */
  private async refresh(): Promise<void> {
    try {
      this.adoptRefreshed(await levelsCommands.restoreLevel(this.session.tool));
    } catch (error: unknown) {
      this.log.warn("Could not read the level again:", transformError(error));
    }
  }

  /**
   * Shows a level read again where it is still the one shown and no older than it; an open or close in flight decides
   * what is shown instead.
   */
  @BoundAction()
  private adoptRefreshed(answer: Nullable<SessionSnapshot<SelectedLevelDescription>>): void {
    const open: Maybe<SessionSnapshot<SelectedLevelDescription>> = this.level.value?.selected;

    if (
      answer &&
      open &&
      !this.level.isLoading &&
      answer.sessionId === open.sessionId &&
      !isOlderSessionAnswer(open, answer)
    ) {
      this.level = this.level.asReady({ selected: answer });
      this.isStale = false;
      this.log.info("Level read again:", describeLevelSource(answer.value.source), `revision ${answer.revision}`);
    }
  }

  /**
   * Reads the parts of an adopted opening the page shows beside its start, side by side, each settling on its own; a
   * part that fails is told by its resource.
   */
  private async attachParts(sessionId: SessionId): Promise<void> {
    await Promise.all([
      this.levelMap.attach(sessionId),
      this.surfaces.attach(sessionId),
      this.textures.attach(sessionId),
      this.spawnObjects.attach(sessionId),
    ]);
  }

  private detachParts(): void {
    this.start.detach();
    this.levelMap.detach();
    this.surfaces.detach();
    this.textures.detach();
    this.spawnObjects.detach();
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
   * Tells a listener each time the open level's scene moves on, streamed afresh, until stopped.
   *
   * @param listener - Hears the scene the level now streams as, or null for none.
   * @returns What stops it.
   */
  public followScene(listener: (scene: Nullable<ILevelScene>) => void): () => void {
    return reaction(() => this.scene, listener);
  }

  private clearView(): void {
    runInAction(() => {
      this.heard = null;
      this.detachParts();
      this.level = this.level.asIdle();
      this.scene = null;
      this.isStale = false;
    });
  }
}
