import { Nullable } from "@xrf/types";
import { ComputeNode, CustomBlending, NodeMaterial, OneFactor, QuadMesh } from "three/webgpu";

import { toLightsPassFragment } from "#/pass/lights-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { createLightBinning } from "#/scene/lights/light-binning.tsl";
import { LIGHT_VECTORS, MAX_LIGHTS, SceneLights } from "#/scene/lights/scene-lights";
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
  private readonly binning: ComputeNode;
  private readonly quad: QuadMesh = new QuadMesh();
  private material: Nullable<NodeMaterial> = null;
  /** The projectors' version its material samples. */
  private version: number = -1;

  /**
   * @param lights - The scene's lights, written out for the frame before this draws.
   * @param targets - The frame's targets: the G-buffer read, the light accumulation added to.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(lights: SceneLights, targets: RendererTargets, uniforms: RendererUniforms) {
    this.lights = lights;
    this.targets = targets;
    this.uniforms = uniforms;
    this.binning = createLightBinning(lights, lights.uniforms, LIGHT_VECTORS, MAX_LIGHTS);
  }

  public render({ renderer, targets }: IRendererFrame): void {
    if (this.lights.count === 0) {
      return;
    }

    renderer.compute(this.binning);
    this.quad.material = this.getMaterial();
    renderer.setRenderTarget(targets.light);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.binning.dispose();
    this.material?.dispose();
  }

  /** The accumulation, built again once the projectors it samples are bound again. */
  private getMaterial(): NodeMaterial {
    if (this.material && this.version === this.lights.version) {
      return this.material;
    }

    this.material?.dispose();
    this.version = this.lights.version;
    this.material = createQuadMaterial(
      toLightsPassFragment(
        {
          counts: this.lights.counts,
          gbuffer: this.targets,
          items: this.lights.items,
          lut: this.uniforms.lut,
          projectors: this.lights.projectors,
          records: this.lights.records,
        },
        this.uniforms.camera,
        this.lights.uniforms,
        LIGHT_VECTORS,
        MAX_LIGHTS
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
