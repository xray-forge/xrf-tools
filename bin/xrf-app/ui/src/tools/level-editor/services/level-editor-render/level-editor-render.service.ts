import { inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, Computed, reaction } from "@wirestate/mobx";
import { cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SceneDrawDescription, SceneHelpersDescription, SelectedSceneDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Sphere } from "@xrf/ipc/types/xrf-math";
import { RenderLoadReport, RenderViewOptions, RenderViewportId } from "@xrf/ipc/types/xrf-renderer";
import { SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-session";
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

/** The open scene's drawing and helpers parts as last read, which the camera is framed by. */
interface ILevelEditorSceneExtent {
  drawing: Nullable<SessionSnapshot<SceneDrawDescription>>;
  helpers: Nullable<SessionSnapshot<SceneHelpersDescription>>;
}

/**
 * Owns the native viewport the open scene is drawn in, under the editor's light, with the Show action's helpers and
 * the hidden objects left out.
 *
 * The scene stays concealed until resident whole. The fly camera frames each opened scene on its drawn objects, or its
 * helpers where it draws none, and one object on request.
 */
@Injectable()
export class LevelEditorRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /**
   * The renderer's texture and model failures for the open scene once resident whole, as for a level; idle until then.
   * A scene has no sectors, so no surfaces are measured.
   */
  public readonly resident: CommandOperation<ILevelResident> = new CommandOperation(this.log);

  /** The fly camera, flown with the toolbar's speeds and lens. */
  public readonly camera: LevelRenderCamera;

  public readonly picks: LevelEditorRenderPicks;

  /** Which renderer scene shows the open scene's snapshot, and whether it is revealed. */
  private readonly shown: LevelSceneShow = new LevelSceneShow({
    onLeft: () => this.leaveScene(),
    onResident: () => void this.describeResident(),
  });

  private sessionId: Nullable<SessionId> = null;
  /** The session the camera was framed on, so a scene is framed once however often its parts are read. */
  private framed: Nullable<SessionId> = null;
  /** The camera's last reported pose, whose heading framing keeps. */
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
        // A decompiled scene's camera stands at the origin, so the scene is framed once its extent is read.
        toStart: () => toLevelStartViewpoint(null),
        viewService,
      }
    );
    this.picks = new LevelEditorRenderPicks(
      { getSessionId: () => this.sessionId, getViewport: () => this.viewport },
      { helpersService, sceneService, selectionService }
    );
  }

  /** Whether the viewport shows the open scene: only once everything it draws is resident, textures included. */
  @Computed()
  public get isRevealed(): boolean {
    return this.shown.isRevealed;
  }

  /**
   * The scene's view options: the settings' features under the editor's light, with helpers and targets as switched.
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
   * Frames a renderer-space sphere, keeping the camera's heading, as `F` frames the selection.
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

  /** Resets state so a remounted view shows and frames the open scene as a first mount does. */
  protected release(): void {
    this.sessionId = null;
    this.framed = null;
    this.pose = null;
    this.camera.release();
    this.shown.release();
  }

  /**
   * Shows the open scene's snapshot, or nothing once it closes.
   *
   * A new session starts from the camera's start; a re-read in the same session replaces the shown snapshot, concealed
   * until ready, since the backend draws a snapshot as shown and follows no commit itself.
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
   * Frames a session's scene once, on its drawn objects or else its helpers. Later re-reads leave the camera where it
   * has flown.
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
   * Asks for texture loads and draw problems once the scene is resident whole. Letting go of the scene cancels it.
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
   * Drops the resident description and picks of the scene shown, which name it alone.
   */
  @BoundAction()
  private leaveScene(): void {
    cancelFlow(this, "resident");
    this.picks.leave();
    this.resident.reset();
  }
}
