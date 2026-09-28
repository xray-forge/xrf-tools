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

import { RENDERER_MAX_SHADOW_CASCADES } from "#/contract/renderer-shadow-settings";
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
   * What the water is prepared in, in one quad: the depth behind it, each pixel's distance along the view in metres
   * as the engine's `s_position.z` holds it, and the distortion target cleared to nothing. Allocated while watered.
   */
  public readonly waterPrepare: RenderTarget;
  /**
   * The frame and the distortion target, with the G-buffer's depth attached: the water composites over the one and
   * writes what it distorts into the other in the same draw. Allocated while watered.
   */
  public readonly water: RenderTarget;
  /**
   * Each shadow cascade's map, its depth alone, reversed like every other: a texel across while its cascade does not
   * draw, so the sun always binds the same textures and a cascade that is off costs nothing.
   */
  public readonly shadows: ReadonlyArray<RenderTarget> = Array.from(
    { length: RENDERER_MAX_SHADOW_CASCADES },
    (_: unknown, view: number) => {
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

  /** Whether the frame draws water, whose targets the next sizing allocates or frees. */
  private isWatered: boolean = false;
  /** Whether the water's targets are allocated now. */
  private isWaterAllocated: boolean = false;

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
    this.waterPrepare = new RenderTarget(1, 1, { count: 2, depthBuffer: false });
    this.waterPrepare.textures[0].name = "water-depth";
    this.waterPrepare.textures[0].format = RedFormat;
    this.waterPrepare.textures[0].type = FloatType;
    this.waterPrepare.textures[1].name = "distortion";

    for (const texture of this.waterPrepare.textures) {
      texture.minFilter = NearestFilter;
      texture.magFilter = NearestFilter;
      texture.generateMipmaps = false;
    }

    this.water = new RenderTarget(1, 1, { count: 2, depthBuffer: true });
    this.water.textures.forEach((texture: Texture) => texture.dispose());
    this.water.textures[0] = this.scene.texture;
    this.water.textures[1] = this.waterPrepare.textures[1];
    this.water.depthTexture = this.gbuffer.depthTexture;
  }

  /** The depth behind the water, in metres along the view. */
  /** Whether the targets have to be allocated again before the frame draws: the water's joined or left since. */
  public get isStale(): boolean {
    return this.isWatered !== this.isWaterAllocated;
  }

  public get waterDepth(): Texture {
    return this.waterPrepare.textures[0];
  }

  /** How far the water moves what is seen through it: `rt_Generic_1`, a half where nothing is moved. */
  public get distortion(): Texture {
    return this.waterPrepare.textures[1];
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

    // The water's frame shares the frame's texture, so it joins or leaves only with every target over that texture
    // freed: allocated or freed alone, it reallocates the texture under targets still holding views of the old one.
    if (this.isStale) {
      [this.scene, this.composite, this.waterPrepare, this.water].forEach((target: RenderTarget) => target.dispose());
    }

    this.gbuffer.setSize(renderWidth, renderHeight);
    this.wallmarks.setSize(renderWidth, renderHeight);
    this.backgroundMotion.setSize(renderWidth, renderHeight);
    this.light.setSize(renderWidth, renderHeight);
    this.scene.setSize(renderWidth, renderHeight);
    this.composite.setSize(renderWidth, renderHeight);
    this.waterPrepare.setSize(renderWidth, renderHeight);
    this.water.setSize(renderWidth, renderHeight);

    renderer.initRenderTarget(this.gbuffer);

    initPreservedDepthTarget(renderer, this.wallmarks);

    renderer.initRenderTarget(this.backgroundMotion);

    renderer.initRenderTarget(this.light);
    renderer.initRenderTarget(this.scene);

    initPreservedDepthTarget(renderer, this.composite);

    if (this.isWatered) {
      renderer.initRenderTarget(this.waterPrepare);
      initPreservedDepthTarget(renderer, this.water);
    }

    this.isWaterAllocated = this.isWatered;
  }

  /**
   * @param isWatered - Whether the frame draws water from its next sizing, which allocates its targets or frees them.
   */
  public setWatered(isWatered: boolean): void {
    this.isWatered = isWatered;
  }

  public dispose(): void {
    [
      this.gbuffer,
      this.wallmarks,
      this.backgroundMotion,
      this.light,
      this.scene,
      this.composite,
      this.waterPrepare,
      this.water,
      this.lightShadows,
      this.lightShadowsStill,
      ...this.shadows,
    ].forEach((target: RenderTarget) => target.dispose());
  }
}
