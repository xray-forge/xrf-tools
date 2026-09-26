import { Nullable } from "@xrf/types";
import { CustomBlending, NodeMaterial, OneFactor, QuadMesh, Texture } from "three/webgpu";

import { ERendererLightShadowFilter } from "#/contract/renderer-features";
import { toLightsPassFragment } from "#/pass/lights-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
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
  private readonly quad: QuadMesh = new QuadMesh();
  private material: Nullable<NodeMaterial> = null;
  /** The projectors' version its material samples, and the filter its shadows are compared through. */
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

  public render({ renderer, targets }: IRendererFrame): void {
    if (this.lights.count === 0) {
      return;
    }

    this.lights.clusters.bin(renderer);
    this.quad.material = this.getMaterial();
    renderer.setRenderTarget(targets.light);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material?.dispose();
  }

  /** The accumulation, built again once the projectors it samples are bound again, or its filter changes. */
  private getMaterial(): NodeMaterial {
    const { clusters, projectors, records } = this.lights;

    if (this.material && this.projectorVersion === projectors.version && this.builtFilter === this.filter) {
      return this.material;
    }

    this.material?.dispose();
    this.projectorVersion = projectors.version;
    this.builtFilter = this.filter;
    this.material = createQuadMaterial(
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
    this.material.blending = CustomBlending;
    this.material.blendSrc = OneFactor;
    this.material.blendDst = OneFactor;
    this.material.blendSrcAlpha = OneFactor;
    this.material.blendDstAlpha = OneFactor;
    this.material.transparent = true;

    return this.material;
  }
}
