import { Maybe } from "@xrf/types";
import { screenUV, vec3, vec4 } from "three/tsl";
import { Node, NodeMaterial, QuadMesh, RenderTarget, TextureNode, WebGPURenderer } from "three/webgpu";

import { ERendererBumpPlane } from "#/contract/renderer-capture";
import { decodeBumpGloss, decodeBumpHeight, decodeBumpNormal } from "#/graph/bump.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { getFlatBumpCompanionTexture, getFlatBumpTexture } from "#/scene/placeholder-textures";
import { RendererTextures } from "#/scene/renderer-textures";

/** Plane materials kept between captures, so a panel redrawing its tiles on every resize compiles nothing. */
const CACHE_LIMIT: number = 16;

/** One plane's material, with the samplers it bound so letting it go unbinds them. */
interface IPlaneMaterial {
  material: NodeMaterial;
  samplers: Array<[string, TextureNode]>;
}

/**
 * Draws one plane of a bump pair face on, texel for texel, through the same decode the surfaces shade with.
 */
export class BumpPlaneCapture {
  private readonly quad: QuadMesh = new QuadMesh();
  private readonly materials: Map<string, IPlaneMaterial> = new Map();
  private readonly textures: RendererTextures;

  public constructor(textures: RendererTextures) {
    this.textures = textures;
  }

  /**
   * @param renderer - The renderer drawing.
   * @param plane - The plane wanted.
   * @param bump - The key of the pair's first half.
   * @param companion - The key of its companion.
   * @param target - Where it is drawn.
   */
  public draw(
    renderer: WebGPURenderer,
    plane: ERendererBumpPlane,
    bump: string,
    companion: string,
    target: RenderTarget
  ): void {
    this.quad.material = this.getMaterial(plane, bump, companion).material;
    renderer.setRenderTarget(target);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.materials.forEach((entry: IPlaneMaterial) => this.release(entry));
    this.materials.clear();
  }

  private getMaterial(plane: ERendererBumpPlane, bump: string, companion: string): IPlaneMaterial {
    const key: string = `${plane}\n${bump}\n${companion}`;
    const cached: Maybe<IPlaneMaterial> = this.materials.get(key);

    if (cached) {
      // Moved to the back, so the cache forgets the plane looked at longest ago.
      this.materials.delete(key);
      this.materials.set(key, cached);

      return cached;
    }

    const bumpSampler: TextureNode = this.textures.bind(bump, getFlatBumpTexture(), screenUV);
    const companionSampler: TextureNode = this.textures.bind(companion, getFlatBumpCompanionTexture(), screenUV);
    const entry: IPlaneMaterial = {
      material: createQuadMaterial(vec4(BumpPlaneCapture.describe(plane, bumpSampler, companionSampler), 1)),
      samplers: [
        [bump, bumpSampler],
        [companion, companionSampler],
      ],
    };

    this.materials.set(key, entry);

    if (this.materials.size > CACHE_LIMIT) {
      const [oldest, evicted] = this.materials.entries().next().value as [string, IPlaneMaterial];

      this.materials.delete(oldest);
      this.release(evicted);
    }

    return entry;
  }

  private release(entry: IPlaneMaterial): void {
    entry.samplers.forEach(([key, sampler]) => this.textures.unbind(key, sampler));
    entry.material.dispose();
  }

  /** What one plane shows, from one texel of each half. */
  private static describe(plane: ERendererBumpPlane, bump: TextureNode, companion: TextureNode): Node<"vec3"> {
    switch (plane) {
      case ERendererBumpPlane.BUMP:
        return bump.xyz;

      case ERendererBumpPlane.COMPANION:
        return companion.xyz;

      case ERendererBumpPlane.NORMAL:
        return decodeBumpNormal(bump, companion).mul(0.5).add(0.5);

      case ERendererBumpPlane.GLOSS:
        return vec3(decodeBumpGloss(bump));

      case ERendererBumpPlane.HEIGHT:
        return vec3(decodeBumpHeight(companion));
    }
  }
}
