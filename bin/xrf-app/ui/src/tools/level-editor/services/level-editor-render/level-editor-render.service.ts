import { inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, Computed, reaction } from "@wirestate/mobx";
import { cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { SceneDrawDescription, SceneHelpersDescription, SelectedSceneDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Sphere } from "@xrf/ipc/types/xrf-math";
import { RenderLoadReport, RenderViewOptions, RenderViewportId } from "@xrf/ipc/types/xrf-renderer";
import { VisualBounds } from "@xrf/ipc/types/xrf-visual";
import { WorldCameraPose, WorldToggles } from "@xrf/ipc/types/xrf-world";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { toLevelFramedGoTo } from "@/core/level/lib/camera/level-camera-frame";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelScene } from "@/core/level/lib/load/level-scene";
import { DEFAULT_LEVEL_RENDER_CONFIG } from "@/core/level/lib/render/level-render-config";
import { ILevelResident, toLevelResident } from "@/core/level/lib/render/level-resident";
import { LevelSceneShow } from "@/core/level/lib/render/level-scene-show";
import { LevelRenderCamera } from "@/core/level/services/level-render/level-render.service.camera";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/viewport/native-render-surface-service";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";
import { INativeViewportClick } from "@/core/render/lib/native/viewport/native-viewport-target";
import { toRenderOutput } from "@/core/render/lib/settings/device/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { toLevelEditorHelperRenderOptions } from "@/tools/level-editor/lib/helpers/level-editor-helper-options";
import {
  DEFAULT_LEVEL_EDITOR_LIGHTING,
  toLevelEditorSceneBounds,
  toLevelEditorSceneSphere,
  toLevelEditorViewOptions,
  toLevelEditorWorldToggles,
} from "@/tools/level-editor/lib/render/level-editor-render";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorRenderPicks } from "@/tools/level-editor/services/level-editor-render/level-editor-render.service.picks";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

/** What the open scene's camera is framed by: its picture's part and its helpers' part, each as last read. */
interface ILevelEditorSceneExtent {
  drawing: Nullable<SessionSnapshot<SceneDrawDescription>>;
  helpers: Nullable<SessionSnapshot<SceneHelpersDescription>>;
}

/**
 * Owns the native viewport the open scene is drawn in: the scene shown, concealed until it is resident whole, what it
 * is asked of the scene once resident, the editor's light in place of a weather, the helpers switched in the Show
 * action, the objects hidden one by one, what a click selects and the pointer hovers (`picks`), and the fly camera,
 * framed on the scene's drawn objects, or its helpers where it draws none, as each scene opens, and on one object as
 * asked.
 */
@Injectable()
export class LevelEditorRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /**
   * What the renderer says of the open scene once it is resident whole, its textures and the library objects it could
   * not make a model of, as it says them of a level: idle until then, and failed with why where it could not say. A
   * scene has no sectors, so nothing of its surfaces is measured.
   */
  public readonly resident: CommandOperation<ILevelResident> = new CommandOperation(this.log);

  /** Where the camera stands, framed on the scene and flown on from with the toolbar's speeds and lens. */
  public readonly camera: LevelRenderCamera;

  /** What of the scene is drawn where the viewport is clicked or hovered, and what that selects and lifts. */
  public readonly picks: LevelEditorRenderPicks;

  /** Which scene shows the open scene's snapshot, and whether it is revealed. */
  private readonly shown: LevelSceneShow = new LevelSceneShow({
    onLeft: () => this.leaveScene(),
    onResident: () => void this.describeResident(),
  });

  /** The scene session shown, or null for none. */
  private sessionId: Nullable<SessionId> = null;
  /** The scene session the camera was framed on, so a scene is framed once however often its picture is read. */
  private framed: Nullable<SessionId> = null;
  /** Where the camera stands and faces as the viewport last said, which framing keeps the heading of. */
  private pose: Nullable<ILevelCamera> = null;

  public constructor(
    private readonly sceneService: LevelEditorSceneService = inject(LevelEditorSceneService),
    private readonly helpersService: LevelEditorHelpersService = inject(LevelEditorHelpersService),
    private readonly selectionService: LevelEditorSelectionService = inject(LevelEditorSelectionService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {
    super(renderSettingsService);

    this.camera = new LevelRenderCamera(
      { getViewport: () => this.viewport },
      {
        config: DEFAULT_LEVEL_RENDER_CONFIG,
        // A decompiled scene's own camera stands at the origin: the scene is framed once its extent is read.
        toStart: () => toLevelStartViewpoint(null),
        viewService,
      }
    );
    this.picks = new LevelEditorRenderPicks(
      { getSessionId: () => this.sessionId, getViewport: () => this.viewport },
      { helpersService, sceneService, selectionService }
    );
  }

  /** Whether the viewport shows the open scene: not until every object it draws is resident, textures included. */
  @Computed()
  public get isRevealed(): boolean {
    return this.shown.isRevealed;
  }

  /**
   * What the scene is asked to be drawn with: the settings' features under the editor's light, helpers as switched, the
   * selected, hovered and hidden objects' singled out.
   */
  @Computed()
  public get options(): RenderViewOptions {
    const { renderSettingsService, viewService, helpersService, selectionService } = this;
    const { hidden, hovered, selectedId } = selectionService;

    return toLevelEditorViewOptions({
      features: renderSettingsService.rendererFeatures,
      helpers: toLevelEditorHelperRenderOptions(helpersService.helpers, { hidden, hovered, selected: selectedId }),
      lighting: DEFAULT_LEVEL_EDITOR_LIGHTING,
      output: toRenderOutput(renderSettingsService.renderResolution, renderSettingsService.renderOutputScale),
      surfaces: viewService.options,
    });
  }

  /**
   * Stands the camera framing a sphere, keeping the heading it faces now, as `F` frames the selection.
   *
   * @param sphere - What to frame, in renderer space.
   */
  public frameSphere(sphere: Sphere): void {
    this.camera.goTo(
      toLevelFramedGoTo(this.pose, toLevelEditorSceneSphere(sphere), this.viewService.cameraOptions.fieldOfView)
    );
  }

  protected start(viewport: NativeViewport): Array<() => void> {
    return [
      reaction(
        () => this.options,
        (options: RenderViewOptions) => viewport.setViewOptions(options),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        () =>
          toLevelEditorWorldToggles({
            hidden: this.selectionService.hidden,
            helpers: this.helpersService.helpers,
            isCollisionShown: this.helpersService.isCollisionShown,
            isEditorOnlyShown: this.helpersService.isEditorOnlyShown,
          }),
        (toggles: WorldToggles) => viewport.setWorldToggles(toggles),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): Nullable<ILevelScene> => {
          const selected: Nullable<SessionSnapshot<SelectedSceneDescription>> = this.sceneService.scene.snapshot;

          return selected ? { revision: selected.revision, sessionId: selected.sessionId } : null;
        },
        this.showScene,
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): ILevelEditorSceneExtent => ({
          drawing: this.sceneService.drawing.snapshot,
          helpers: this.sceneService.helpers.snapshot,
        }),
        this.frameOnce,
        { equals: comparer.shallow, fireImmediately: true }
      ),
      ...this.camera.start(),
      ...this.picks.start(viewport),
    ];
  }

  protected onCamera(pose: WorldCameraPose): void {
    this.pose = toLevelCameraReading(pose);
  }

  protected onLoad(report: RenderLoadReport): void {
    this.viewportService.noteLoad(report);
    this.shown.noteLoad(report);
  }

  protected onClick(click: INativeViewportClick): void {
    this.picks.click(click);
  }

  protected onHover(point: Nullable<IRenderViewPoint>): void {
    this.picks.rest(point);
  }

  protected onFlySpeed(steps: number): void {
    this.camera.stepSpeed(steps);
  }

  /** A view mounted again shows and frames the scene open as a view first shown does. */
  protected release(): void {
    this.sessionId = null;
    this.framed = null;
    this.pose = null;
    this.camera.release();
    this.shown.release();
  }

  /**
   * Shows the open scene's snapshot: a scene opened anew from the camera's start, one read again in its session in
   * place of the snapshot shown, concealed until it is ready, since the backend draws a snapshot as it was shown and
   * follows no commit itself; and nothing once the scene closes.
   *
   * @param scene - The open scene's session and the revision its snapshot was read at, or null for none open.
   */
  @BoundAction()
  private showScene(scene: Nullable<ILevelScene>): void {
    const sessionId: Nullable<SessionId> = scene?.sessionId ?? null;

    if (this.shown.open(sessionId, scene)) {
      this.camera.standAtStart();
      this.viewportService.noteLoad(null);
    } else {
      this.shown.move(scene);
    }

    this.sessionId = sessionId;
    this.show(
      (viewport: NativeViewport) =>
        viewport.ask("showScene", (id: RenderViewportId) => levelsCommands.showScene(id, sessionId), null),
      this.shown.noteShown
    );
  }

  /**
   * Frames the camera on the drawn objects of a scene the first time its picture is read with any, or on its helpers
   * once they are read where it draws none: reading either again, as the scene's load finishes, leaves the camera where
   * it has flown.
   *
   * @param extent - What the open scene's picture and helpers need, each as last read, or null before it is.
   */
  @BoundAction()
  private frameOnce(extent: ILevelEditorSceneExtent): void {
    const { sessionId, framed } = this;
    const { drawing, helpers } = extent;
    const bounds: Nullable<VisualBounds> = toLevelEditorSceneBounds(
      drawing?.sessionId === sessionId ? drawing.value : null,
      helpers?.sessionId === sessionId ? helpers.value : null
    );

    if (!bounds || sessionId === null || sessionId === framed) {
      return;
    }

    this.framed = sessionId;
    this.camera.goTo(
      toLevelFramedGoTo(
        null,
        toLevelEditorSceneSphere(bounds.boundingSphere),
        this.viewService.cameraOptions.fieldOfView
      )
    );
  }

  /**
   * Asks what each texture came to and what the scene could not draw, once it is resident whole; the scene shown let go
   * meanwhile cancels it, as it describes something else.
   */
  @LatestFlow("resident")
  private *describeResident(): TFlow {
    const viewport: Nullable<NativeViewport> = this.viewport;

    if (!viewport) {
      return;
    }

    yield* this.resident.run({
      name: "Describe resident scene",
      command: () =>
        Promise.all([viewport.describeTextureLoads(), viewport.describeProblems()]).then(([textures, problems]) =>
          toLevelResident([], textures, problems)
        ),
      failure: "Could not describe the resident scene",
    });
  }

  /**
   * Lets go of what was asked of the scene shown: what it described and what a click picked or the pointer hovered in
   * it name it alone.
   */
  @BoundAction()
  private leaveScene(): void {
    cancelFlow(this, "resident");
    this.picks.leave();
    this.resident.reset();
  }
}
