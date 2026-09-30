import { Nullable } from "@xrf/types";
import { CustomBlending, NodeMaterial, OneFactor, Texture } from "three/webgpu";

import { ERendererLightShadowFilter } from "#/contract/renderer-light-shadow-filter";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { toLightsPassFragment } from "#/pass/lights-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { SceneLights } from "#/scene/lights/scene-lights";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * The local lights, after the sun: those standing in view binned into the clusters of the view, then every pixel lit
 * by the ones its cluster holds, added to what the sun accumulated. Costs nothing while no light stands in view.
 */
export class LightsPass implements IRendererPass {
  public readonly name: string = "lights";

  private readonly lights: SceneLights;
  private readonly targets: RendererTargets;
  private readonly uniforms: RendererUniforms;
  private draw: Nullable<FullScreenDraw> = null;
  /** The projectors' version its draw samples, and the filter its shadows are compared through. */
  private projectorVersion: number = -1;
  private filter: ERendererLightShadowFilter = ERendererLightShadowFilter.ENGINE;
  private builtFilter: ERendererLightShadowFilter = ERendererLightShadowFilter.ENGINE;

  /**
   * @param lights - The scene's lights, written out for the frame before this draws.
   * @param targets - The frame's targets: the G-buffer read, the light accumulation added to.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(lights: SceneLights, targets: RendererTargets, uniforms: RendererUniforms) {
    this.lights = lights;
    this.targets = targets;
    this.uniforms = uniforms;
  }

  /**
   * @param filter - How the shadows are filtered from the next frame, which builds the accumulation again.
   */
  public setFilter(filter: ERendererLightShadowFilter): void {
    this.filter = filter;
  }

  /** Named whether or not a light stands in view, so the first to come draws at once. */
  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.compute(this.lights.clusters.kernels);
    pipelines.draw(this.getDraw());
  }

  public render({ renderer }: IRendererFrame): void {
    if (this.lights.count === 0) {
      return;
    }

    this.lights.clusters.bin(renderer);
    this.getDraw().render(renderer);
  }

  public dispose(): void {
    this.draw?.dispose();
  }

  /** The accumulation, built again once the projectors it samples are bound again, or its filter changes. */
  private getDraw(): FullScreenDraw {
    const { clusters, projectors, records } = this.lights;

    if (this.draw && this.projectorVersion === projectors.version && this.builtFilter === this.filter) {
      return this.draw;
    }

    this.draw?.dispose();
    this.projectorVersion = projectors.version;
    this.builtFilter = this.filter;

    const material: NodeMaterial = createQuadMaterial(
      toLightsPassFragment(
        {
          atlas: this.targets.lightShadows.depthTexture as Texture,
          counts: clusters.counts,
          gbuffer: this.targets,
          items: clusters.items,
          lut: this.uniforms.lut,
          projectors: projectors.samplers,
          records: records.buffer,
        },
        this.uniforms.camera,
        clusters.uniforms,
        this.filter
      )
    );

    // Added to the sun, colour and specular alike: `blend(true, D3DBLEND_ONE, D3DBLEND_ONE)`.
    material.blending = CustomBlending;
    material.blendSrc = OneFactor;
    material.blendDst = OneFactor;
    material.blendSrcAlpha = OneFactor;
    material.blendDstAlpha = OneFactor;
    material.transparent = true;
    this.draw = new FullScreenDraw(material, this.targets.light);

    return this.draw;
  }
}
