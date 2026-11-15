import { EventBus, inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, RefObservable } from "@wirestate/mobx";
import { call, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SessionPartResource } from "@xrf/ipc/session";
import { SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import {
  SCENE_BINARY_TOOLS_PART,
  SCENE_DRAW_PART,
  SCENE_OBJECTS_PART,
  SceneBinaryToolsDescription,
  SceneDrawDescription,
  SceneObjectsDescription,
  SceneOpenRequest,
  SceneSource,
  SelectedSceneDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";
import { Maybe, Nullable } from "@xrf/types";

import { SessionRootResource } from "@/core/session/lib/session-root-resource";

/**
 * Opens a Level Editor scene, its `.level` and the part files beside it, and keeps each part of it the editor shows -
 * its object tools' objects, its binary tools' counts and what its picture needs - as a resource read apart from it
 * and again apart from it. Nothing of the scene is written.
 */
@Injectable()
export class LevelEditorSceneService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Every object of the open scene's object tools, typed, which the Outliner lists and the Inspector shows. */
  public readonly objects: SessionPartResource<SceneObjectsDescription>;

  /** What the open scene's AI map, detail objects and wall marks hold, counted. */
  public readonly binaryTools: SessionPartResource<SceneBinaryToolsDescription>;

  /**
   * Where the open scene's drawn objects stand, which the camera is framed on, and each scene object it does not draw
   * for its library object missing or unreadable. What the renderer could not compile or texture is the viewport's to
   * say once the scene is resident (`LevelEditorRenderService.resident`).
   */
  public readonly drawing: SessionPartResource<SceneDrawDescription>;

  /**
   * The open scene, read again once its session derives it anew. A scene that fails to open, or to close, is let go
   * all the same: the editor shows the failure rather than the scene it was asked to leave.
   */
  public readonly scene: SessionRootResource<SelectedSceneDescription> = new SessionRootResource(this.log, {
    subject: "scene",
    close: levelsCommands.closeScene,
    restore: levelsCommands.restoreScene,
    follow: { reread: levelsCommands.restoreScene },
    isLetGoOnFailure: true,
    adopted: (selected: SessionSnapshot<SelectedSceneDescription>) => this.attachParts(selected.sessionId),
  });

  /** The last open asked for, which a retry asks for again. */
  private request: Nullable<SceneOpenRequest> = null;

  /** The scene being opened, named before it is open: what the editor heads its workspace with while it waits. */
  @RefObservable()
  public opening: Nullable<SceneSource> = null;

  /** Opens begun, so one taken over by a later open leaves that one's name. */
  private openings: number = 0;

  public constructor(eventBus: EventBus = inject(EventBus)) {
    this.objects = new SessionPartResource(eventBus, [SCENE_OBJECTS_PART], levelsCommands.describeSceneObjects);
    this.binaryTools = new SessionPartResource(
      eventBus,
      [SCENE_BINARY_TOOLS_PART],
      levelsCommands.describeSceneBinaryTools
    );
    this.drawing = new SessionPartResource(eventBus, [SCENE_DRAW_PART], levelsCommands.describeSceneDraw);
  }

  /** Whether a change moved what the open scene was read from: it shows as it opened until it is read again. */
  @Computed()
  public get isStale(): boolean {
    return this.scene.isStale;
  }

  /**
   * Takes back whatever scene the backend still has open.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await this.restore();
  }

  /**
   * Restores the scene the backend holds for the editor unless a user action has taken the scene's flow, its parts
   * read after it.
   */
  @BoundAction()
  public restore(): Promise<void> {
    return flowResult(this.scene.restore());
  }

  /**
   * Opens a scene and lists its tools' part files, its parts read after it. One that fails closes the scene open before
   * it, so the editor shows the failure rather than the scene it was asked to replace.
   *
   * @param request - The `.level` to open and the roots its library objects are looked for in.
   */
  @BoundAction()
  public async open(request: SceneOpenRequest): Promise<void> {
    const opening: number = ++this.openings;

    this.request = request;
    this.opening = request.source;

    try {
      const isOpened: Maybe<boolean> = await flowResult(
        this.scene.open(request.source.path, levelsCommands.openScene, request)
      );

      // Failed rather than superseded: a later open's answer decides what the parts read.
      if (isOpened === false) {
        this.detachParts();
      }
    } finally {
      // A later open took the flow over meanwhile, and names its own scene.
      this.forgetOpening(opening);
    }
  }

  /**
   * Opens the last scene asked for again, after it failed.
   */
  public async retry(): Promise<void> {
    if (this.request) {
      await this.open(this.request);
    }
  }

  /**
   * Closes the open scene without clearing a newer one. The view is cleared even where the backend failed to let the
   * scene go, which stays owned for the next close to release.
   */
  @BoundAction()
  public async close(): Promise<void> {
    const isClosed: Maybe<boolean> = await flowResult(this.scene.close());

    // Superseded rather than settled: a later open's answer decides what the parts read.
    if (isClosed !== undefined) {
      this.detachParts();
    }
  }

  /**
   * Abandons pending work and releases the scene at once, for a surface leaving while its tool stays.
   */
  @BoundAction()
  public clear(): void {
    this.scene.release();
    this.detachParts();
  }

  @BoundAction()
  private forgetOpening(opening: number): void {
    if (opening === this.openings) {
      this.opening = null;
    }
  }

  /**
   * Reads the parts of an adopted opening side by side, each settling on its own; a part that fails is told by its
   * resource. Attaching to another opening lets the parts of the scene held before go first, so nothing shows one
   * scene's parts beside another.
   */
  private *attachParts(sessionId: SessionId): TFlow {
    yield* call(
      Promise.all([this.objects.attach(sessionId), this.binaryTools.attach(sessionId), this.drawing.attach(sessionId)])
    );
  }

  @BoundAction()
  private detachParts(): void {
    this.objects.detach();
    this.binaryTools.detach();
    this.drawing.detach();
  }
}
