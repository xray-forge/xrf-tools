import { HalfFloatType, NearestFilter, NodeMaterial, QuadMesh, RenderTarget, RGFormat, Texture } from "three/webgpu";

import { ERendererAmbientOcclusionQuality } from "#/contract/renderer-features";
import {
  IAmbientOcclusionSearch,
  toAmbientOcclusionDenoise,
  toAmbientOcclusionSearch,
} from "#/pass/ambient-occlusion-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { AmbientOcclusionUniforms } from "#/uniforms/ambient-occlusion-uniforms";
import { CameraUniforms } from "#/uniforms/camera-uniforms";

/** XeGTAO's quality presets: directions around the view, and steps each way along each. */
const AMBIENT_OCCLUSION_SEARCHES: Readonly<Record<ERendererAmbientOcclusionQuality, IAmbientOcclusionSearch>> = {
  [ERendererAmbientOcclusionQuality.LOW]: { slices: 1, steps: 2 },
  [ERendererAmbientOcclusionQuality.MEDIUM]: { slices: 2, steps: 2 },
  [ERendererAmbientOcclusionQuality.HIGH]: { slices: 3, steps: 3 },
  [ERendererAmbientOcclusionQuality.ULTRA]: { slices: 6, steps: 3 },
};

/** A half resolution target of visibility and distance, read texel by texel. */
function createAmbientOcclusionTarget(name: string): RenderTarget {
  const target: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false, format: RGFormat, type: HalfFloatType });

  target.texture.name = name;
  target.texture.minFilter = NearestFilter;
  target.texture.magFilter = NearestFilter;
  target.texture.generateMipmaps = false;

  return target;
}

/**
 * GTAO: the frame's depth searched for what occludes each point at half resolution, then denoised one way and the
 * other, for combine to bring up to the frame's size. In the frame only while the occlusion is on, so off it costs
 * nothing; its two targets go with it.
 */
export class AmbientOcclusionPass implements IRendererPass {
  public readonly name: string = "gtao";

  private readonly quad: QuadMesh = new QuadMesh();
  private readonly uniforms: AmbientOcclusionUniforms = new AmbientOcclusionUniforms();
  /** What the search and the second way of the denoise write, and combine reads. */
  private readonly searched: RenderTarget = createAmbientOcclusionTarget("gtao");
  /** What the first way of the denoise writes. */
  private readonly denoised: RenderTarget = createAmbientOcclusionTarget("gtao-denoise");
  private readonly search: NodeMaterial;
  private readonly across: NodeMaterial;
  private readonly down: NodeMaterial;

  /**
   * @param quality - How many directions and steps each pixel searches.
   * @param targets - The frame's targets, whose G-buffer is searched.
   * @param camera - The drawing camera's uniforms.
   */
  public constructor(quality: ERendererAmbientOcclusionQuality, targets: RendererTargets, camera: CameraUniforms) {
    this.search = createQuadMaterial(
      toAmbientOcclusionSearch(targets, camera, this.uniforms, AMBIENT_OCCLUSION_SEARCHES[quality])
    );
    this.across = createQuadMaterial(toAmbientOcclusionDenoise(this.searched.texture, [1, 0]));
    this.down = createQuadMaterial(toAmbientOcclusionDenoise(this.denoised.texture, [0, 1]));
  }

  /** The occlusion at half resolution: visibility in red, distance along the view in green. */
  public get output(): Texture {
    return this.searched.texture;
  }

  public render({ renderer, camera, targets, settings }: IRendererFrame): void {
    const width: number = Math.ceil(targets.gbuffer.width / 2);
    const height: number = Math.ceil(targets.gbuffer.height / 2);

    this.searched.setSize(width, height);
    this.denoised.setSize(width, height);
    this.uniforms.take(settings.features.ambientOcclusion, camera, width, height);

    for (const [material, target] of [
      [this.search, this.searched],
      [this.across, this.denoised],
      [this.down, this.searched],
    ] as const) {
      this.quad.material = material;
      renderer.setRenderTarget(target);
      this.quad.render(renderer);
    }
  }

  public dispose(): void {
    [this.search, this.across, this.down].forEach((material: NodeMaterial) => material.dispose());
    this.searched.dispose();
    this.denoised.dispose();
  }
}
