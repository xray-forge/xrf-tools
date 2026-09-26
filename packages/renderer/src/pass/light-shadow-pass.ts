import { NodeMaterial, QuadMesh, RenderTarget } from "three/webgpu";

import { toFarDepth, toNoColor } from "#/pass/light-shadow-pass.tsl";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { ILightShadowFace, LIGHT_SHADOW_ATLAS_SIZE, LightShadows } from "#/scene/lights/light-shadows";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { STATIC_LIGHT_VIEW_START } from "#/uniforms/static-draw-buffers";

/**
 * Culls the light faces queued this frame as one batch, each into its own reusable shadow-view slot, then draws each
 * into its square of the atlas. Only that square is cleared: clearing the target would erase the cached faces. In the
 * frame only while the lights draw shadows, the atlas a texel across otherwise.
 */
export class LightShadowPass implements IRendererPass {
  public readonly name: string = "light-shadows";

  private readonly shadows: LightShadows;
  private readonly target: RenderTarget;
  private readonly casters: IStaticShadowCasters;
  private readonly cull: StaticCull;
  private readonly clear: QuadMesh;
  private readonly clearMaterial: NodeMaterial = new NodeMaterial();

  /**
   * @param shadows - What plans the faces.
   * @param targets - The frame's targets, whose atlas the faces are drawn into.
   * @param casters - What the shadow views draw.
   * @param cull - What culls the static draws, including the scheduled light faces.
   */
  public constructor(shadows: LightShadows, targets: RendererTargets, casters: IStaticShadowCasters, cull: StaticCull) {
    this.shadows = shadows;
    this.target = targets.lightShadows;
    this.casters = casters;
    this.cull = cull;
    this.target.setSize(LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_ATLAS_SIZE);
    this.clearMaterial.fragmentNode = toNoColor();
    this.clearMaterial.depthNode = toFarDepth();
    // Written whatever it stands over: three turns `AlwaysDepth` into `NeverDepth` for a reversed depth buffer, so
    // the test is turned off instead, which compares always and still writes.
    this.clearMaterial.depthTest = false;
    this.clearMaterial.depthWrite = true;
    this.clearMaterial.colorWrite = false;
    this.clear = new QuadMesh(this.clearMaterial);
  }

  public render({ renderer }: IRendererFrame): void {
    const faces: ReadonlyArray<ILightShadowFace> = this.shadows.queue;

    if (!faces.length) {
      return;
    }

    this.cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    renderer.sortObjects = false;

    for (const [index, face] of faces.entries()) {
      const { x, y, size } = face.tile;
      const view: number = STATIC_LIGHT_VIEW_START + index;

      this.casters.showShadowCells(view, face.planes);
      this.target.viewport.set(x, y, size, size);
      renderer.setRenderTarget(this.target);
      this.clear.render(renderer);
      renderer.render(this.casters.shadowScenes[view], face.camera);

      if (this.casters.plainCasters.children.length > 0) {
        renderer.render(this.casters.plainCasters, face.camera);
      }
    }

    renderer.sortObjects = true;
    this.target.viewport.set(0, 0, LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_ATLAS_SIZE);
    this.shadows.markDrawn();
  }

  /** Gives the atlas back to a texel, and every face with it. */
  public dispose(): void {
    this.target.setSize(1, 1);
    this.shadows.reset();
    this.clearMaterial.dispose();
  }
}
