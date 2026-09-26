import { Nullable } from "@xrf/types";
import { NodeMaterial, PerspectiveCamera, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { toFarDepth, toNoColor } from "#/pass/light-shadow-pass.tsl";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { drawUnsorted } from "#/pass/unsorted-draw";
import { ILightShadowFace, LIGHT_SHADOW_ATLAS_SIZE, LightShadowPlanner } from "#/scene/lights/light-shadow-planner";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { STATIC_LIGHT_VIEW_START } from "#/uniforms/static-draw-buffers";

/**
 * Culls the light faces queued this frame as one batch, each into its own shadow-view slot, then draws each into its
 * square of the atlas through one camera that takes each face's matrices, so a slot's recordings replay for every face
 * it draws. Only that square is cleared: clearing the target would erase the faces kept. In the frame only while the
 * lights draw shadows; the atlas is a texel across otherwise.
 */
export class LightShadowPass implements IRendererPass {
  public readonly name: string = "light-shadows";

  private readonly planner: LightShadowPlanner;
  private readonly target: RenderTarget;
  private readonly casters: IStaticShadowCasters;
  private readonly cull: StaticCull;
  private readonly camera: PerspectiveCamera = new PerspectiveCamera();
  private readonly clear: QuadMesh;
  private readonly clearMaterial: NodeMaterial = new NodeMaterial();
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
    this.casters = casters;
    this.cull = cull;
    adoptRendererConventions(this.camera);
    // Its matrices are a face's, copied whole before each draw.
    this.camera.matrixAutoUpdate = false;
    this.camera.matrixWorldAutoUpdate = false;
    this.clearMaterial.fragmentNode = toNoColor();
    this.clearMaterial.depthNode = toFarDepth();
    // Written whatever it stands over: three turns `AlwaysDepth` into `NeverDepth` for a reversed depth buffer, so
    // the test is turned off instead, which compares always and still writes.
    this.clearMaterial.depthTest = false;
    this.clearMaterial.depthWrite = true;
    this.clearMaterial.colorWrite = false;
    this.clear = new QuadMesh(this.clearMaterial);
  }

  /** Allocates the atlas for a renderer, whatever the frame's size: a new one holds no face, so each is drawn again. */
  public resize(renderer: WebGPURenderer): void {
    if (renderer === this.renderer) {
      return;
    }

    this.renderer = renderer;
    this.target.setSize(LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_ATLAS_SIZE);
    renderer.initRenderTarget(this.target);
    this.planner.forgetDrawn();
  }

  public render({ renderer }: IRendererFrame): void {
    const faces: ReadonlyArray<ILightShadowFace> = this.planner.queue;

    if (!faces.length) {
      return;
    }

    this.cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    drawUnsorted(renderer, () =>
      faces.forEach((face: ILightShadowFace, index: number) => this.draw(renderer, face, index))
    );
    this.target.viewport.set(0, 0, LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_ATLAS_SIZE);
    this.planner.markDrawn();
  }

  /** Gives the atlas back to a texel; the faces keep their squares, drawn again once the atlas is. */
  public dispose(): void {
    this.target.setSize(1, 1);
    this.clearMaterial.dispose();
  }

  /** Draws a face into its square, through the light-view camera taking its matrices, from its view slot. */
  private draw(renderer: WebGPURenderer, face: ILightShadowFace, index: number): void {
    const { x, y, size } = face.tile;
    const view: number = STATIC_LIGHT_VIEW_START + index;

    this.takeFace(face);
    this.casters.showShadowCells(view, face.planes);
    this.target.viewport.set(x, y, size, size);
    renderer.setRenderTarget(this.target);
    this.clear.render(renderer);
    renderer.render(this.casters.shadowScenes[view], this.camera);

    if (this.casters.plainCasters.show(face.planes)) {
      renderer.render(this.casters.plainCasters.scene, this.camera);
    }
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
