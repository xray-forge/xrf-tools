import { Maybe } from "@xrf/types";
import { NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { toBumpPlaneFragment } from "#/capture/bump-plane-capture.tsl";
import { ERendererBumpPlane } from "#/contract/renderer-capture";
import { MaterialSamplers } from "#/material/material-samplers";
import { createQuadMaterial } from "#/pass/quad-material";
import { RendererTextures } from "#/texture/renderer-textures";

/** Plane materials kept between captures, so a panel redrawing its tiles on every resize compiles nothing. */
const CACHE_LIMIT: number = 16;

/** One plane's material, with the samplers it bound so letting it go unbinds them. */
interface IPlaneMaterial {
  material: NodeMaterial;
  samplers: MaterialSamplers;
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
    this.materials.forEach(BumpPlaneCapture.release);
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

    const samplers: MaterialSamplers = new MaterialSamplers(this.textures);
    const entry: IPlaneMaterial = {
      material: createQuadMaterial(toBumpPlaneFragment(plane, samplers, bump, companion)),
      samplers,
    };

    this.materials.set(key, entry);

    if (this.materials.size > CACHE_LIMIT) {
      const [oldest, evicted] = this.materials.entries().next().value as [string, IPlaneMaterial];

      this.materials.delete(oldest);
      BumpPlaneCapture.release(evicted);
    }

    return entry;
  }

  private static release(entry: IPlaneMaterial): void {
    entry.samplers.release();
    entry.material.dispose();
  }
}
