import { EventBus, inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, RefObservable } from "@wirestate/mobx";
import { call, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SessionPartResource } from "@xrf/ipc/session";
import {
  SCENE_BINARY_TOOLS_PART,
  SCENE_DRAW_PART,
  SCENE_HELPERS_PART,
  SCENE_OBJECTS_PART,
  SceneBinaryToolsDescription,
  SceneDrawDescription,
  SceneHelpersDescription,
  SceneObjectsDescription,
  SceneOpenRequest,
  SceneSource,
  SelectedSceneDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-session";
import { Logger } from "@xrf/logging";
import { Maybe, Nullable } from "@xrf/types";

import { SessionRootResource } from "@/core/session/lib/session-root-resource";

/**
 * Opens a Level Editor scene read-only: its `.level` and part files, with each part the editor shows kept as a
 * resource read and re-read on its own.
 */
@Injectable()
export class LevelEditorSceneService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Every object of the open scene's object tools, for the Outliner and the Inspector. */
  public readonly objects: SessionPartResource<SceneObjectsDescription>;

  /** The counts of the open scene's AI map, detail objects and wall marks. */
  public readonly binaryTools: SessionPartResource<SceneBinaryToolsDescription>;

  /**
   * The drawn objects' extent, which the camera frames, and the scene objects not drawn for a missing or unreadable
   * library object. Compile and texture failures come from `LevelEditorRenderService.resident` instead.
   */
  public readonly drawing: SessionPartResource<SceneDrawDescription>;

  /** The helpers' extent, framed where the scene draws no object, and each helped tool's count. */
  public readonly helpers: SessionPartResource<SceneHelpersDescription>;

  /**
   * The open scene, re-read when its session derives it anew. A scene that fails to open or close is let go anyway, so
   * the editor shows the failure rather than the scene it was asked to leave.
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

  /** The scene being opened, which heads the workspace while it waits. */
  @RefObservable()
  public opening: Nullable<SceneSource> = null;

  /** Opens begun, so a superseded open leaves the later one's name. */
  private openings: number = 0;

  public constructor(eventBus: EventBus = inject(EventBus)) {
    this.objects = new SessionPartResource(eventBus, [SCENE_OBJECTS_PART], levelsCommands.describeSceneObjects);
    this.binaryTools = new SessionPartResource(
      eventBus,
      [SCENE_BINARY_TOOLS_PART],
      levelsCommands.describeSceneBinaryTools
    );
    this.drawing = new SessionPartResource(eventBus, [SCENE_DRAW_PART], levelsCommands.describeSceneDraw);
    this.helpers = new SessionPartResource(eventBus, [SCENE_HELPERS_PART], levelsCommands.describeSceneHelpers);
  }

  /** Whether the open scene's sources changed; it shows as opened until read again. */
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
   * Restores the scene the backend holds, then its parts, unless a user action has taken the scene's flow.
   */
  @BoundAction()
  public restore(): Promise<void> {
    return flowResult(this.scene.restore());
  }

  /**
   * Opens a scene, then reads its parts. A failed open closes the previous scene, so the editor shows the failure
   * rather than the scene it was asked to replace.
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

      // Only a failure detaches; a superseded open leaves the parts to the later one.
      if (isOpened === false) {
        this.detachParts();
      }
    } finally {
      // Unless a later open took over and names its own scene.
      this.forgetOpening(opening);
    }
  }

  /**
   * Retries the last open, or the restore where nothing was opened since.
   */
  public async retry(): Promise<void> {
    if (this.request) {
      await this.open(this.request);
    } else {
      await this.restore();
    }
  }

  /**
   * Closes the open scene without clearing a newer one. The view clears even where the backend failed to let go; the
   * scene then stays owned for the next close.
   */
  @BoundAction()
  public async close(): Promise<void> {
    const isClosed: Maybe<boolean> = await flowResult(this.scene.close());

    // Undefined means superseded: the later open's answer decides what the parts read.
    if (isClosed !== undefined) {
      this.detachParts();
    }
  }

  @BoundAction()
  private forgetOpening(opening: number): void {
    if (opening === this.openings) {
      this.opening = null;
    }
  }

  /**
   * Reads an adopted session's parts in parallel, each settling and failing on its own. The previous scene's parts are
   * let go first, so no two scenes' parts show together.
   */
  private *attachParts(sessionId: SessionId): TFlow {
    yield* call(
      Promise.all([
        this.objects.attach(sessionId),
        this.binaryTools.attach(sessionId),
        this.drawing.attach(sessionId),
        this.helpers.attach(sessionId),
      ])
    );
  }

  @BoundAction()
  private detachParts(): void {
    this.objects.detach();
    this.binaryTools.detach();
    this.drawing.detach();
    this.helpers.detach();
  }
}
