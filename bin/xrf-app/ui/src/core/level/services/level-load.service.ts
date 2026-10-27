import { EventBus, inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, reaction, RefObservable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { call, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { isSessionMovedPast, SessionPartResource } from "@xrf/ipc/session";
import { SessionChange, SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
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
} from "@xrf/ipc/types/xrf-app-levels";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { XrayRevision } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Maybe, Nullable } from "@xrf/types";

import { describeLevelSource } from "@/core/level/lib/source/level-source";
import { EMPTY_LEVEL_SPAWN_REPORT, ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";

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

  /**
   * The open level, read again once its session derives it anew. A level that fails to open, or to close, is let go
   * all the same: the viewer shows the failure rather than the level it was asked to leave.
   */
  public readonly selection: SessionRootResource<SelectedLevelDescription> = new SessionRootResource(this.log, {
    subject: "level",
    close: levelsCommands.closeLevel,
    restore: levelsCommands.restoreLevel,
    follow: { reread: levelsCommands.restoreLevel, heard: (change: SessionChange) => this.noteScene(change) },
    isLetGoOnFailure: true,
    adopting: (selected: SessionSnapshot<SelectedLevelDescription>) => this.meet(selected),
    adopted: (selected: SessionSnapshot<SelectedLevelDescription>) => this.attachParts(selected.sessionId),
  });

  /** The last open asked for, which a retry asks for again. */
  private request: Nullable<LevelOpenRequest> = null;

  /** The level being opened, named before it is open: what a viewer heads its viewport with while it waits. */
  @RefObservable()
  public opening: Nullable<LevelSource> = null;

  /** Opens begun, so one taken over by a later open leaves that one's name. */
  private openings: number = 0;

  /**
   * The scene the open level streams as, moved on each time the backend streams it afresh: the level derived anew, or
   * what its sectors are packed and drawn from changed. Kept with the opening it streams, so a level let go has none.
   */
  @RefObservable()
  private streamed: Nullable<ILevelScene> = null;

  public constructor(eventBus: EventBus = inject(EventBus)) {
    this.start = new SessionPartResource(eventBus, [LEVEL_START_PART], levelsCommands.describeStart);
    this.levelMap = new SessionPartResource(eventBus, [LEVEL_MAP_PART], levelsCommands.describeLevelMap);
    this.surfaces = new SessionPartResource(eventBus, [LEVEL_SURFACES_PART], levelsCommands.describeSurfaces);
    this.textures = new SessionPartResource(eventBus, [LEVEL_TEXTURES_PART], levelsCommands.describeTextures);
    this.spawnObjects = new SessionPartResource(eventBus, [LEVEL_SPAWN_PART], levelsCommands.describeSpawnObjects);
  }

  /** The open level, its lifecycle and failure kept. */
  @Computed()
  public get level(): AsyncState<IOpenLevel> {
    return this.selection.mapSnapshot((selected: SessionSnapshot<SelectedLevelDescription>) => ({ selected }));
  }

  /** The scene the open level streams as, or none while no level is open. */
  @Computed()
  public get scene(): Nullable<ILevelScene> {
    return this.streamed?.sessionId === this.selection.sessionId ? this.streamed : null;
  }

  /** Whether a change moved what the open level was read from: it shows as it opened until it is read again. */
  @Computed()
  public get isStale(): boolean {
    return this.selection.isStale;
  }

  /** Whether the restore has settled, one way or the other. */
  @Computed()
  public get isReady(): boolean {
    return this.selection.isRestored;
  }

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

  /**
   * Takes back whatever level the backend still has open.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await this.restore();
  }

  /**
   * Restores the native selection unless a user action has taken the level flow: where it starts read before it shows,
   * the rest of its parts after.
   */
  @BoundAction()
  public restore(): Promise<void> {
    return flowResult(this.selection.restore());
  }

  /**
   * Open a level and report what it is built out of: where it starts read before it shows, the rest of its parts
   * after. One that fails closes the level open before it, so the viewer shows the failure rather than the level it was
   * asked to replace.
   *
   * @param request - The level to open, the roots it is searched in, and how the game's configs beside it are read.
   */
  @BoundAction()
  public async load(request: LevelOpenRequest): Promise<void> {
    const opening: number = ++this.openings;

    this.request = request;
    this.opening = request.source;

    try {
      const isOpened: Maybe<boolean> = await flowResult(
        this.selection.open(describeLevelSource(request.source), levelsCommands.openLevel, request)
      );

      // Failed rather than superseded: a later open's answer decides what the parts read.
      if (isOpened === false) {
        this.detachParts();
      }
    } finally {
      // A later open took the flow over meanwhile, and names its own level.
      this.forgetOpening(opening);
    }
  }

  /**
   * Opens the last level asked for again, after it failed.
   */
  public async retry(): Promise<void> {
    if (this.request) {
      await this.load(this.request);
    }
  }

  /**
   * Closes this loader's level without clearing a newer one. The view is cleared even where the backend failed to let
   * the level go, which stays owned for the next close to release.
   */
  @BoundAction()
  public async close(): Promise<void> {
    const isClosed: Maybe<boolean> = await flowResult(this.selection.close());

    // Superseded rather than settled: a later open's answer decides what the parts read.
    if (isClosed !== undefined) {
      this.detachParts();
    }
  }

  /**
   * Abandons pending work and releases the level at once, for a surface leaving while its tool stays.
   */
  @BoundAction()
  public clear(): void {
    this.selection.release();
    this.detachParts();
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

  @BoundAction()
  private forgetOpening(opening: number): void {
    if (opening === this.openings) {
      this.opening = null;
    }
  }

  /**
   * Reads where an opening starts before it shows, then takes the scene it streams as with it, letting the parts of the
   * level held before go at once, so nothing shows one level's parts beside another.
   *
   * @param selected - The opening the backend holds, from an open or a restore.
   */
  private *meet(selected: SessionSnapshot<SelectedLevelDescription>): TFlow {
    yield* call(this.start.attach(selected.sessionId));

    this.streamed = { revision: selected.revision, sessionId: selected.sessionId };
    this.levelMap.detach();
    this.surfaces.detach();
    this.textures.detach();
    this.spawnObjects.detach();
  }

  /** Moves the scene on for a change past the one it streams as that streams it afresh, as the backend says. */
  @BoundAction()
  private noteScene(change: SessionChange): void {
    if (this.streamed && change.isViewRenewed && isSessionMovedPast(this.streamed, change)) {
      this.streamed = { revision: change.revision, sessionId: change.sessionId };
    }
  }

  /**
   * Reads the parts of an adopted opening the page shows beside its start, side by side, each settling on its own; a
   * part that fails is told by its resource.
   */
  private *attachParts(sessionId: SessionId): TFlow {
    yield* call(
      Promise.all([
        this.levelMap.attach(sessionId),
        this.surfaces.attach(sessionId),
        this.textures.attach(sessionId),
        this.spawnObjects.attach(sessionId),
      ])
    );
  }

  @BoundAction()
  private detachParts(): void {
    this.start.detach();
    this.levelMap.detach();
    this.surfaces.detach();
    this.textures.detach();
    this.spawnObjects.detach();
  }
}
