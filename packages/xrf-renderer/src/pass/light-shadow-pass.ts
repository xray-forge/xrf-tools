import { Nullable } from "@xrf/types";
import {
  DepthTexture,
  Node,
  NodeMaterial,
  Object3D,
  PerspectiveCamera,
  RenderTarget,
  Scene,
  WebGPURenderer,
} from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { drawTogether } from "#/pass/drawn-together";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { toFarDepth, toKeptDepth, toNoColor } from "#/pass/light-shadow-pass.tsl";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { LIGHT_SHADOW_ATLAS_SIZE } from "#/scene/lights/light-shadow-atlas";
import { ILightShadowFace } from "#/scene/lights/light-shadow-face";
import { LightShadowPlanner } from "#/scene/lights/light-shadow-planner";
import { createSceneRoot } from "#/scene/object/scene-mesh";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { LIGHT_SHADOW_FACE_BUDGET } from "#/uniforms/lights-uniforms";
import { STATIC_LIGHT_VIEW_START } from "#/uniforms/static-draw-buffers";

/**
 * Culls the light faces queued this frame as one batch, each into its own shadow-view slot, then draws each into its
 * square of the atlas through one camera that takes each face's matrices, so a slot's recordings replay for every face
 * it draws. Only that square is cleared: clearing the target would erase the faces kept. What stands still is drawn
 * into an atlas of its own and kept there, so a face drawn again only because what sways moved starts from what it
 * kept and draws the swaying and the plain casters alone, in one render call with the copy. In the frame only while the
 * lights draw shadows; the atlases are a texel across otherwise.
 */
export class LightShadowPass implements IRendererPass {
  public readonly name: string = "light-shadows";

  private readonly planner: LightShadowPlanner;
  private readonly target: RenderTarget;
  private readonly still: RenderTarget;
  private readonly casters: IStaticShadowCasters;
  private readonly cull: StaticCull;
  private readonly camera: PerspectiveCamera = new PerspectiveCamera();
  /** Clears a face's square of the still atlas to the far depth. */
  private readonly clear: FullScreenDraw;
  /** Writes what a face kept of what stands still into its square, as its draw over what sways starts. */
  private readonly keep: FullScreenDraw;
  /** What a face's square is drawn from in one render call: the clear or the copy, then the casters. */
  private readonly holder: Scene = createSceneRoot();
  /** What the holder draws next, an array reused by every face. */
  private readonly parts: Array<Object3D> = [];
  /** The renderer the atlas was allocated by: another starts it out holding nothing. */
  private renderer: Nullable<WebGPURenderer> = null;

  /**
   * @param planner - What plans the faces.
   * @param targets - The frame's targets, whose atlas the faces are drawn into.
   * @param casters - What the shadow views draw.
   * @param cull - What culls the static draws, the queued faces among them.
   */
  public constructor(
    planner: LightShadowPlanner,
    targets: RendererTargets,
    casters: IStaticShadowCasters,
    cull: StaticCull
  ) {
    this.planner = planner;
    this.target = targets.lightShadows;
    this.still = targets.lightShadowsStill;
    this.casters = casters;
    this.cull = cull;
    adoptRendererConventions(this.camera);
    // Its matrices are a face's, copied whole before each draw.
    this.camera.matrixAutoUpdate = false;
    this.camera.matrixWorldAutoUpdate = false;
    this.clear = new FullScreenDraw(createDepthOnlyMaterial(toFarDepth()), this.still);
    this.keep = new FullScreenDraw(
      createDepthOnlyMaterial(toKeptDepth(this.still.depthTexture as DepthTexture)),
      this.target
    );
  }

  /** Allocates the atlas for a renderer, whatever the frame's size: a new one holds no face, so each is drawn again. */
  public resize(renderer: WebGPURenderer): void {
    if (renderer === this.renderer) {
      return;
    }

    this.renderer = renderer;
    // Both sized before either is allocated: they share a colour, which sizing frees.
    this.target.setSize(LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_ATLAS_SIZE);
    this.still.setSize(LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_ATLAS_SIZE);
    renderer.initRenderTarget(this.target);
    renderer.initRenderTarget(this.still);
    this.planner.forgetDrawn();
  }

  /** Every face slot's cull, whether or not a face is queued, so the first light to cast draws at once. */
  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.draw(this.clear);
    pipelines.draw(this.keep);

    for (let index: number = 0; index < LIGHT_SHADOW_FACE_BUDGET; index += 1) {
      pipelines.compute(this.cull.getViewKernels(STATIC_LIGHT_VIEW_START + index));
    }
  }

  public render({ renderer }: IRendererFrame): void {
    const faces: ReadonlyArray<ILightShadowFace> = this.planner.queue;

    if (!faces.length) {
      return;
    }

    this.cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    faces.forEach((face: ILightShadowFace, index: number) => this.draw(renderer, face, index));
    this.target.viewport.set(0, 0, LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_ATLAS_SIZE);
    this.still.viewport.set(0, 0, LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_ATLAS_SIZE);
    this.planner.markDrawn();
  }

  /** Gives the atlas back to a texel; the faces keep their squares, drawn again once the atlas is. */
  public dispose(): void {
    this.target.setSize(1, 1);
    this.still.setSize(1, 1);
    this.clear.dispose();
    this.keep.dispose();
  }

  /**
   * Draws a face into its square, through the light-view camera taking its matrices, from its view slot: what stands
   * still into the still atlas where that changed or was never drawn, then that into the atlas and what sways over it.
   * Each atlas's square is one render call, its full screen draw first: their corners ignore the camera.
   */
  private draw(renderer: WebGPURenderer, face: ILightShadowFace, index: number): void {
    const { x, y, size } = face.tile;
    const view: number = STATIC_LIGHT_VIEW_START + index;
    // Drawn, and nothing it keeps changed: it is queued for what sways or moves alone.
    const isStillKept: boolean = face.isDrawn && !face.isStale;
    const { holder, parts } = this;

    this.takeFace(face);

    if (!isStillKept) {
      parts.length = 0;
      parts.push(this.clear.object);
      pushIfShown(parts, this.casters.stillShadowScenes[view]);
      this.still.viewport.set(x, y, size, size);
      renderer.setRenderTarget(this.still);
      drawTogether(renderer, holder, parts, this.camera);
    }

    parts.length = 0;
    parts.push(this.keep.object);
    pushIfShown(parts, this.casters.swayingShadowScenes[view]);

    if (this.casters.plainCasters.show(face.planes)) {
      parts.push(this.casters.plainCasters.scene);
    }

    this.target.viewport.set(x, y, size, size);
    renderer.setRenderTarget(this.target);
    drawTogether(renderer, holder, parts, this.camera);
  }

  private takeFace(face: ILightShadowFace): void {
    const { camera } = this;

    camera.near = face.near;
    camera.far = face.far;
    camera.matrixWorld.copy(face.world);
    camera.matrixWorldInverse.copy(face.view);
    camera.projectionMatrix.copy(face.projection);
    camera.projectionMatrixInverse.copy(face.projection).invert();
  }
}

/**
 * A full screen material writing a depth of its own at every pixel and no colour, whatever stood there: the test is
 * left off, since three turns `AlwaysDepth` into `NeverDepth` for a reversed depth buffer, and so it compares always
 * and still writes.
 *
 * @param depth - The depth each pixel writes.
 * @returns The material.
 */
function createDepthOnlyMaterial(depth: Node<"float">): NodeMaterial {
  const material: NodeMaterial = new NodeMaterial();

  material.fragmentNode = toNoColor();
  material.depthNode = depth;
  material.depthTest = false;
  material.depthWrite = true;
  material.colorWrite = false;

  return material;
}

/**
 * Adds a shadow scene to what a face draws where any of its bundles is shown to the view: each costs its setup whatever
 * it draws, and a face often reaches nothing that sways.
 */
function pushIfShown(parts: Array<Object3D>, scene: Scene): void {
  if (scene.children.some((child: Object3D) => child.visible)) {
    parts.push(scene);
  }
}
