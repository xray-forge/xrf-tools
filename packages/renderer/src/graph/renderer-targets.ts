import {
  DepthTexture,
  FloatType,
  HalfFloatType,
  NearestFilter,
  RenderTarget,
  RGFormat,
  Texture,
  WebGPURenderer,
} from "three/webgpu";

/** The G-buffer attachments, by the names the materials' `mrt` writes them under. */
export enum EGBufferTarget {
  /** Raw albedo in colour, gloss in alpha: byte for byte the engine's `rt_Color`. */
  ALBEDO = "albedo",
  /** The octahedral view space normal. */
  NORMAL = "normal",
  /** Hemisphere occlusion, sun occlusion, lighting model slice, flags. */
  SURFACE = "surface",
}

/**
 * Every target a frame draws into, sized together.
 */
export class RendererTargets {
  /** `rgba8unorm`, `rg16float`, `rgba8unorm`, and `depth32float`: sixteen bytes a pixel. */
  public readonly gbuffer: RenderTarget;
  /**
   * The albedo alone with the G-buffer's depth, which wall marks composite into before any light: `phase_wallmarks`
   * binds `rt_Color` and nothing else.
   */
  public readonly wallmarks: RenderTarget;
  /** What the lights accumulate: diffuse in colour, specular in alpha. */
  public readonly light: RenderTarget;
  /** The tonemapped frame, as combine writes it: no depth, since combine samples the G-buffer's. */
  public readonly scene: RenderTarget;
  /**
   * The same frame with the G-buffer's depth attached, so composited surfaces are hidden by what stands in front.
   * A second target over one texture, because WebGPU refuses a depth both sampled and attached in one pass.
   */
  public readonly composite: RenderTarget;

  public constructor() {
    this.gbuffer = new RenderTarget(1, 1, { count: 3, depthBuffer: true });
    this.gbuffer.textures[0].name = EGBufferTarget.ALBEDO;
    this.gbuffer.textures[1].name = EGBufferTarget.NORMAL;
    this.gbuffer.textures[1].format = RGFormat;
    this.gbuffer.textures[1].type = HalfFloatType;
    this.gbuffer.textures[2].name = EGBufferTarget.SURFACE;
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

  public get depth(): DepthTexture {
    return this.gbuffer.depthTexture as DepthTexture;
  }

  /**
   * @param width - Drawing buffer width, in device pixels.
   * @param height - Drawing buffer height, in device pixels.
   */
  public resize(width: number, height: number): void {
    this.gbuffer.setSize(width, height);
    this.wallmarks.setSize(width, height);
    this.light.setSize(width, height);
    this.scene.setSize(width, height);
    this.composite.setSize(width, height);
  }

  /**
   * Allocates every target at its size before a pass draws into one.
   *
   * Three allocates a target on first use, and allocating one that shares a texture reallocates it: left to the pass
   * drawing into `composite` or `wallmarks`, that would erase what the pass before drew on every frame after a resize.
   *
   * @param renderer - The renderer the targets are drawn by.
   */
  public prepare(renderer: WebGPURenderer): void {
    renderer.initRenderTarget(this.gbuffer);
    renderer.initRenderTarget(this.wallmarks);
    renderer.initRenderTarget(this.light);
    renderer.initRenderTarget(this.scene);
    renderer.initRenderTarget(this.composite);
  }

  public dispose(): void {
    this.gbuffer.dispose();
    this.wallmarks.dispose();
    this.light.dispose();
    this.scene.dispose();
    this.composite.dispose();
  }
}
