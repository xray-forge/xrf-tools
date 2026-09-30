import { Maybe } from "@xrf/types";
import { RenderTarget, WebGPURenderer } from "three/webgpu";

import { toBumpPlaneFragment } from "#/capture/bump-plane-capture.tsl";
import { ERendererBumpPlane } from "#/contract/renderer-bump-plane";
import { MaterialSamplers } from "#/material/material-samplers";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { RendererTextures } from "#/texture/renderer-textures";

/** Plane materials kept between captures, so a panel redrawing its tiles on every resize compiles nothing. */
const CACHE_LIMIT: number = 16;

/** One plane's draw, with the samplers its material bound so letting it go unbinds them. */
interface IPlaneDraw {
  draw: FullScreenDraw;
  samplers: MaterialSamplers;
}

/**
 * Draws one plane of a bump pair face on, texel for texel, through the same decode the surfaces shade with.
 */
export class BumpPlaneCapture {
  private static release(entry: IPlaneDraw): void {
    entry.samplers.release();
    entry.draw.dispose();
  }

  private readonly draws: Map<string, IPlaneDraw> = new Map();
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
    this.getDraw(plane, bump, companion, target).draw.render(renderer, target);
  }

  public dispose(): void {
    this.draws.forEach(BumpPlaneCapture.release);
    this.draws.clear();
  }

  /** A plane's draw, made for the target it is first drawn into; every capture's target has the same attachments. */
  private getDraw(plane: ERendererBumpPlane, bump: string, companion: string, target: RenderTarget): IPlaneDraw {
    const key: string = `${plane}\n${bump}\n${companion}`;
    const cached: Maybe<IPlaneDraw> = this.draws.get(key);

    if (cached) {
      // Moved to the back, so the cache forgets the plane looked at longest ago.
      this.draws.delete(key);
      this.draws.set(key, cached);

      return cached;
    }

    const samplers: MaterialSamplers = new MaterialSamplers(this.textures);
    const entry: IPlaneDraw = {
      draw: new FullScreenDraw(createQuadMaterial(toBumpPlaneFragment(plane, samplers, bump, companion)), target),
      samplers,
    };

    this.draws.set(key, entry);

    if (this.draws.size > CACHE_LIMIT) {
      const [oldest, evicted] = this.draws.entries().next().value as [string, IPlaneDraw];

      this.draws.delete(oldest);
      BumpPlaneCapture.release(evicted);
    }

    return entry;
  }
}
