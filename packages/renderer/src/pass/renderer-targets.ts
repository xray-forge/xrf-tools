import {
  DepthTexture,
  FloatType,
  HalfFloatType,
  NearestFilter,
  RedFormat,
  RenderTarget,
  RGFormat,
  Texture,
  WebGPURenderer,
} from "three/webgpu";

import { RENDERER_MAX_SHADOW_CASCADES } from "#/contract/renderer-features";
import { initPreservedDepthTarget } from "#/internals/preserved-depth-target";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
import { IGBufferTextures } from "#/shader/gbuffer-textures";

/**
 * Every target a frame draws into, sized together.
 */
export class RendererTargets implements IGBufferTextures {
  /**
   * `rgba8unorm`, `rg16float`, `rgba8unorm`, `rg16float` and `depth32float`: twenty bytes a pixel. The fourth is the
   * motion every surface writes, which the temporal resolve reprojects its history by.
   */
  public readonly gbuffer: RenderTarget;
  /**
   * The albedo alone with the G-buffer's depth, which wall marks composite into before any light: `phase_wallmarks`
   * binds `rt_Color` and nothing else.
   */
  public readonly wallmarks: RenderTarget;
  /** The motion alone, which the sky's motion is written into where the G-buffer drew nothing. */
  public readonly backgroundMotion: RenderTarget;
  /** What the lights accumulate: diffuse in colour, specular in alpha. */
  public readonly light: RenderTarget;
  /** The tonemapped frame, as combine writes it: no depth, since combine samples the G-buffer's. */
  public readonly scene: RenderTarget;
  /**
   * The same frame with the G-buffer's depth attached, so composited surfaces are hidden by what stands in front.
   * A second target over one texture, because WebGPU refuses a depth both sampled and attached in one pass.
   */
  public readonly composite: RenderTarget;
  /**
   * Each shadow cascade's map, its depth alone, reversed like every other: a texel across while its cascade does not
   * draw, so the sun always binds the same textures and a cascade that is off costs nothing.
   */
  public readonly shadows: ReadonlyArray<RenderTarget> = Array.from(
    { length: RENDERER_MAX_SHADOW_CASCADES },
    (_, view: number) => {
      // The colour a render target cannot go without, as small as a texel can be; nothing writes it.
      const target: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true, format: RedFormat });

      target.texture.name = `shadow-${view}`;
      target.depthTexture = new DepthTexture(1, 1, FloatType);
      target.depthTexture.name = `shadow-depth-${view}`;

      return target;
    }
  );

  /**
   * The lights' shadow atlas, every shadowed light's faces in squares of it, its depth alone read: a texel across while
   * the lights draw no shadows, so the lights always bind the same texture. Three draws into no target without a colour
   * attachment, so it keeps one of a byte a texel, never written.
   */
  public readonly lightShadows: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true, format: RedFormat });
  /**
   * The atlas of what stands still: each face's square as it was last drawn in full, which a face drawn again for what
   * sways starts from. It shares the lights' colour, which nothing writes.
   */
  public readonly lightShadowsStill: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true, format: RedFormat });

  public constructor() {
    this.lightShadows.texture.name = "light-shadows";
    this.lightShadows.depthTexture = new DepthTexture(1, 1, FloatType);
    this.lightShadows.depthTexture.name = "light-shadows-depth";
    this.lightShadowsStill.texture.dispose();
    this.lightShadowsStill.texture = this.lightShadows.texture;
    this.lightShadowsStill.depthTexture = new DepthTexture(1, 1, FloatType);
    this.lightShadowsStill.depthTexture.name = "light-shadows-still-depth";
    this.gbuffer = new RenderTarget(1, 1, { count: 4, depthBuffer: true });
    // Named for the device's labels alone: the surfaces write the attachments by location, in this order.
    this.gbuffer.textures[0].name = "albedo";
    this.gbuffer.textures[1].name = "normal";
    this.gbuffer.textures[1].format = RGFormat;
    this.gbuffer.textures[1].type = HalfFloatType;
    this.gbuffer.textures[2].name = "surface";
    this.gbuffer.textures[3].name = "motion";
    this.gbuffer.textures[3].format = RGFormat;
    this.gbuffer.textures[3].type = HalfFloatType;
    this.gbuffer.depthTexture = new DepthTexture(1, 1, FloatType);

    for (const texture of this.gbuffer.textures) {
      texture.minFilter = NearestFilter;
      texture.magFilter = NearestFilter;
      texture.generateMipmaps = false;
    }

    this.wallmarks = new RenderTarget(1, 1, { depthBuffer: true });
    this.wallmarks.texture.dispose();
    this.wallmarks.texture = this.gbuffer.textures[0];
    this.wallmarks.depthTexture = this.gbuffer.depthTexture;

    this.backgroundMotion = new RenderTarget(1, 1, { depthBuffer: false });
    this.backgroundMotion.texture.dispose();
    this.backgroundMotion.texture = this.gbuffer.textures[3];

    this.light = new RenderTarget(1, 1, { depthBuffer: false, type: HalfFloatType });
    this.scene = new RenderTarget(1, 1, { depthBuffer: false });
    this.composite = new RenderTarget(1, 1, { depthBuffer: true });
    this.composite.texture.dispose();
    this.composite.texture = this.scene.texture;
    // The G-buffer claimed the depth first, so it stays the target three resizes the depth with.
    this.composite.depthTexture = this.gbuffer.depthTexture;
  }

  public get albedo(): Texture {
    return this.gbuffer.textures[0];
  }

  public get normal(): Texture {
    return this.gbuffer.textures[1];
  }

  public get surface(): Texture {
    return this.gbuffer.textures[2];
  }

  /** How far each pixel's surface moved on the screen since the frame before, in texture coordinates. */
  public get motion(): Texture {
    return this.gbuffer.textures[3];
  }

  public get depth(): DepthTexture {
    return this.gbuffer.depthTexture as DepthTexture;
  }

  /**
   * Sizes every target to the scene as drawn and allocates it before a pass draws into one: three allocates a target
   * on first use, and allocating one that shares a texture reallocates it, which left to a pass would erase what the
   * pass before drew. Every size is set before anything is allocated, since sizing a target frees its textures: a
   * target sharing one, sized after the owner was allocated, frees what the owner still draws into.
   *
   * @param renderer - The renderer the targets are drawn by.
   * @param size - The frame's size.
   */
  public resize(renderer: WebGPURenderer, size: IRendererFrameSize): void {
    const { renderWidth, renderHeight } = size;

    this.gbuffer.setSize(renderWidth, renderHeight);
    this.wallmarks.setSize(renderWidth, renderHeight);
    this.backgroundMotion.setSize(renderWidth, renderHeight);
    this.light.setSize(renderWidth, renderHeight);
    this.scene.setSize(renderWidth, renderHeight);
    this.composite.setSize(renderWidth, renderHeight);

    renderer.initRenderTarget(this.gbuffer);

    initPreservedDepthTarget(renderer, this.wallmarks);

    renderer.initRenderTarget(this.backgroundMotion);

    renderer.initRenderTarget(this.light);
    renderer.initRenderTarget(this.scene);

    initPreservedDepthTarget(renderer, this.composite);
  }

  public dispose(): void {
    [
      this.gbuffer,
      this.wallmarks,
      this.backgroundMotion,
      this.light,
      this.scene,
      this.composite,
      this.lightShadows,
      this.lightShadowsStill,
      ...this.shadows,
    ].forEach((target: RenderTarget) => target.dispose());
  }
}
