import { Nullable } from "@xrf/types";
import { PerspectiveCamera, RenderTarget, Vector2, WebGPURenderer } from "three/webgpu";

import {
  ERendererAntialiasing,
  IRendererFeatureSettings,
  isRendererTemporal,
  RENDERER_MAX_SHADOW_CASCADES,
  toRendererUpscale,
} from "#/contract/renderer-features";
import { createBaseFramePasses } from "#/graph/base-frame-passes";
import { AmbientOcclusionPass } from "#/pass/ambient-occlusion-pass";
import { AntialiasPass, toPresentedFrame } from "#/pass/antialias/antialias-pass";
import { CombinePass } from "#/pass/combine-pass";
import { FsrPass } from "#/pass/fsr/fsr-pass";
import { GrassPass } from "#/pass/grass-pass";
import { LightShadowPass } from "#/pass/light-shadow-pass";
import { LightsPass } from "#/pass/lights-pass";
import { OverlayPass } from "#/pass/overlay-pass";
import { PresentPass } from "#/pass/present-pass";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererScenePass, isRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { ShadowPass } from "#/pass/shadow-pass";
import { SharpenPass } from "#/pass/sharpen-pass";
import { SpatialUpscalePass } from "#/pass/spatial-upscale-pass";
import { TemporalAntialiasPass } from "#/pass/temporal-antialias-pass";
import { toRenderSize } from "#/pass/temporal-jitter";
import { ITemporalUpscaler } from "#/pass/temporal-upscaler";
import { SceneGrass } from "#/scene/grass/scene-grass";
import { SceneLights } from "#/scene/lights/scene-lights";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { RendererPassInspector } from "#/timing/renderer-pass-inspector";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * The frame: every target it draws into, and its passes in order, the picture presented last.
 */
export class RendererFrameGraph {
  public readonly targets: RendererTargets = new RendererTargets();
  /** The last pass, which a capture also draws into a target of its own. */
  public readonly present: PresentPass;
  /** The passes drawing the consumer's scenes, whose materials compile against their targets. */
  public readonly scenePasses: ReadonlyArray<IRendererScenePass>;
  /** Every pass's name, in frame order, as the frame report states them. */
  public passNames: ReadonlyArray<string> = [];
  /** The scene's size as drawn: the output's, or smaller while it is upscaled. */
  public readonly renderSize: Vector2 = new Vector2();

  private readonly base: ReadonlyArray<IRendererPass>;
  private passes: ReadonlyArray<IRendererPass> = [];
  /** The pass smoothing the frame's edges, while a mode over the finished frame is chosen. */
  private antialias: Nullable<AntialiasPass> = null;
  /** The pass resolving jittered frames with their history, while TAA or FSR is chosen. */
  private temporal: Nullable<ITemporalUpscaler> = null;
  /** FSR 1, upscaling what the other modes finish, while they draw the scene smaller. */
  private spatial: Nullable<SpatialUpscalePass> = null;
  /** The pass sharpening what was upscaled, while the scene is upscaled and sharpened. */
  private sharpen: Nullable<SharpenPass> = null;
  private antialiasing: ERendererAntialiasing = ERendererAntialiasing.NONE;
  /** The ratio of the output's side to the scene's as drawn. */
  private upscale: number = 1;
  /** The grass, while it is on. */
  private grassPass: Nullable<GrassPass> = null;
  /** The local lights, while they are on. */
  private lightsPass: Nullable<LightsPass> = null;
  /** The lights' shadow faces, while the lights draw shadows. */
  private lightShadowPass: Nullable<LightShadowPass> = null;
  /** The screen's occlusion while it is on, and the quality it was made for. */
  private ambientOcclusion: Nullable<AmbientOcclusionPass> = null;
  private ambientOcclusionKey: string = "";
  /** A pass a shadow cascade drawn, and the cascades and resolution they were made for. */
  private shadows: Array<ShadowPass> = [];
  private shadowKey: string = "";
  private readonly uniforms: RendererUniforms;
  private readonly cull: StaticCull;
  private readonly casters: IStaticShadowCasters;
  private readonly grass: SceneGrass;
  private readonly lights: SceneLights;
  /** The pass the occlusion is combined in. */
  private readonly combine: CombinePass;
  /** The pass the helpers draw in, over the upscaled or resolved frame where there is one. */
  private readonly overlay: OverlayPass;
  /** What the targets were last sized by, for a pass that joins the frame after: the renderer and the output's size. */
  private renderer: Nullable<WebGPURenderer> = null;
  private width: number = 0;
  private height: number = 0;

  /**
   * @param uniforms - What the frame's shaders read.
   * @param overlays - The helpers drawn last.
   * @param cull - What culls the static draws.
   * @param casters - What each shadow cascade draws.
   * @param grass - The level's grass, which the grass pass plants and draws.
   * @param lights - The local lights, which the lights pass bins and accumulates.
   */
  public constructor(
    uniforms: RendererUniforms,
    overlays: RendererOverlays,
    cull: StaticCull,
    casters: IStaticShadowCasters,
    grass: SceneGrass,
    lights: SceneLights
  ) {
    this.uniforms = uniforms;
    this.grass = grass;
    this.lights = lights;
    this.cull = cull;
    this.casters = casters;
    this.present = new PresentPass(this.targets, uniforms.camera);
    this.base = createBaseFramePasses(this.targets, uniforms, overlays, cull);
    this.scenePasses = this.base.filter(isRendererScenePass);
    this.combine = this.base.find((pass: IRendererPass) => pass instanceof CombinePass) as CombinePass;
    this.overlay = this.base.find((pass: IRendererPass) => pass instanceof OverlayPass) as OverlayPass;
    this.link();
  }

  /**
   * Puts into the frame the passes the features want and takes out the ones they do not, whose targets go with them.
   *
   * @param features - What the features are set to.
   */
  public configure(features: IRendererFeatureSettings): void {
    const { shadows } = features;
    const count: number = shadows.isEnabled ? Math.min(shadows.cascades.length, RENDERER_MAX_SHADOW_CASCADES) : 0;
    const shadowKey: string = `${count}:${shadows.resolution}`;
    const ambientOcclusionKey: string = features.ambientOcclusion.isEnabled ? features.ambientOcclusion.quality : "";
    const isLightShadowed: boolean = features.lights.isEnabled && features.lights.isShadowed;
    const upscale: number = toRendererUpscale(features);
    const isSpatial: boolean = !isRendererTemporal(features.antialiasing) && upscale > 1;
    const isSharpened: boolean = upscale > 1 && features.upscaling.sharpening > 0;

    // Its material is built again for another filter; the pass itself stays.
    this.lightsPass?.setFilter(features.lights.shadowFilter);
    this.sharpen?.setSharpening(features.upscaling.sharpening);

    if (
      features.antialiasing === this.antialiasing &&
      upscale === this.upscale &&
      isSpatial === (this.spatial !== null) &&
      isSharpened === (this.sharpen !== null) &&
      shadowKey === this.shadowKey &&
      ambientOcclusionKey === this.ambientOcclusionKey &&
      features.grass.isEnabled === (this.grassPass !== null) &&
      features.lights.isEnabled === (this.lightsPass !== null) &&
      isLightShadowed === (this.lightShadowPass !== null)
    ) {
      return;
    }

    if (isLightShadowed !== (this.lightShadowPass !== null)) {
      this.lightShadowPass?.dispose();
      this.lightShadowPass = isLightShadowed
        ? new LightShadowPass(this.lights.shadows, this.targets, this.casters, this.cull)
        : null;
    }

    if (features.lights.isEnabled !== (this.lightsPass !== null)) {
      this.lightsPass?.dispose();
      this.lightsPass = features.lights.isEnabled ? new LightsPass(this.lights, this.targets, this.uniforms) : null;
    }

    this.lightsPass?.setFilter(features.lights.shadowFilter);

    if (features.grass.isEnabled !== (this.grassPass !== null)) {
      this.grassPass?.dispose();
      this.grassPass = features.grass.isEnabled ? new GrassPass(this.grass, this.targets) : null;
    }

    if (ambientOcclusionKey !== this.ambientOcclusionKey) {
      this.ambientOcclusion?.dispose();
      this.ambientOcclusionKey = ambientOcclusionKey;
      this.ambientOcclusion = features.ambientOcclusion.isEnabled
        ? new AmbientOcclusionPass(features.ambientOcclusion.quality, this.targets, this.uniforms.camera)
        : null;
      this.combine.setAmbientOcclusion(this.ambientOcclusion?.output ?? null);
      this.present.setAmbientOcclusion(this.ambientOcclusion?.output ?? null);
    }

    const isResolved: boolean = features.antialiasing !== this.antialiasing;

    if (isResolved) {
      this.antialias?.dispose();
      this.temporal?.dispose();
      this.antialiasing = features.antialiasing;
      this.antialias = null;
      this.temporal = null;

      if (features.antialiasing === ERendererAntialiasing.TAA) {
        this.temporal = new TemporalAntialiasPass(this.targets, this.uniforms);
      } else if (features.antialiasing === ERendererAntialiasing.FSR) {
        this.temporal = new FsrPass(this.targets, this.uniforms);
      } else if (features.antialiasing !== ERendererAntialiasing.NONE) {
        this.antialias = new AntialiasPass(features.antialiasing, this.targets);
      }
    }

    // It reads what the smoothing finished, so it goes and comes back with a new mode.
    const isRespatial: boolean = isResolved || isSpatial !== (this.spatial !== null);

    if (isRespatial) {
      this.spatial?.dispose();
      this.spatial = isSpatial
        ? new SpatialUpscalePass(toPresentedFrame(this.antialias, this.targets), this.targets, this.uniforms)
        : null;
    }

    const upscaled: Nullable<RenderTarget> = this.temporal?.output ?? this.spatial?.output ?? null;
    // It reads the upscaled output, so it goes and comes back with a new upscaler.
    const isResharpened: boolean = isRespatial || isSharpened !== (this.sharpen !== null);

    if (isResharpened) {
      this.sharpen?.dispose();
      this.sharpen =
        isSharpened && upscaled ? new SharpenPass(upscaled, features.antialiasing === ERendererAntialiasing.FSR) : null;
      this.sharpen?.setSharpening(features.upscaling.sharpening);
    }

    if (isResharpened || upscale !== this.upscale) {
      this.upscale = upscale;
      // Textures sampled as finely as the output shows them, whatever size the scene is drawn at.
      this.uniforms.settings.textureBias.value = -Math.log2(upscale);
      this.applySize();
    }

    const shown: Nullable<RenderTarget> = this.sharpen?.output ?? upscaled;

    this.overlay.setTarget(shown);
    this.present.setFrame(shown ?? toPresentedFrame(this.antialias, this.targets));

    if (shadowKey !== this.shadowKey) {
      this.shadows.forEach((pass: ShadowPass) => pass.dispose());
      this.shadowKey = shadowKey;
      this.shadows = Array.from(
        { length: count },
        (_, view: number) =>
          new ShadowPass(
            view,
            this.targets,
            this.casters,
            this.cull,
            this.uniforms.shadows,
            this.uniforms.wind,
            shadows.resolution
          )
      );
    }

    this.link();
  }

  /**
   * @param renderer - The renderer the targets are drawn by.
   * @param width - Drawing buffer width, in device pixels: the output's.
   * @param height - Drawing buffer height, in device pixels.
   */
  public resize(renderer: WebGPURenderer, width: number, height: number): void {
    this.renderer = renderer;
    this.width = width;
    this.height = height;
    this.applySize();
  }

  /**
   * Offsets the camera's samples within the pixel for this frame while TAA or FSR resolves it, before anything reads
   * the camera; the resolve gives it back its projection before the helpers draw.
   *
   * @param camera - The drawing camera, its projection as its controller left it.
   */
  public jitter(camera: PerspectiveCamera): void {
    this.temporal?.jitter(camera);
  }

  /**
   * @param frame - What the frame draws with.
   * @param inspector - What times each pass under its name.
   */
  public render(frame: IRendererFrame, inspector: RendererPassInspector): void {
    for (const pass of this.passes) {
      inspector.enter(pass.name);
      pass.render(frame);
      inspector.leave();
    }
  }

  public dispose(): void {
    this.passes.forEach((pass: IRendererPass) => pass.dispose());
    this.targets.dispose();
  }

  /** Sizes the scene's targets to the drawing and the resolve's to the output, once there is a renderer to size by. */
  private applySize(): void {
    const { renderer, width, height } = this;

    if (!renderer) {
      return;
    }

    this.renderSize.set(toRenderSize(width, this.upscale), toRenderSize(height, this.upscale));
    this.targets.resize(this.renderSize.x, this.renderSize.y);
    this.targets.prepare(renderer);
    this.temporal?.resize(renderer, width, height, this.renderSize.x, this.renderSize.y);
    this.spatial?.resize(renderer, width, height);
    this.sharpen?.resize(renderer, width, height);
  }

  /**
   * The frame's passes in order: the base's with the shadow cascades before the sun reads them, the local lights after
   * it, the occlusion before combine does, FSR's copy of the opaque frame before the blended surfaces draw, and the
   * upscaling and its sharpening before the helpers, whatever else the features add, then the picture presented. The
   * smoothing of the other modes comes after the helpers, which it smooths too, unless FSR 1 upscales what it smoothed.
   */
  private link(): void {
    const sun: number = this.base.findIndex((pass: IRendererPass) => pass.name === "sun");
    const combine: number = this.base.indexOf(this.combine);
    const forward: number = this.base.findIndex((pass: IRendererPass) => pass.name === "forward");
    const overlay: number = this.base.indexOf(this.overlay);
    const smoothing: ReadonlyArray<IRendererPass> = this.antialias ? [this.antialias] : [];

    const gbuffer: number = this.base.findIndex((pass: IRendererPass) => pass.name === "gbuffer") + 1;

    this.passes = [
      ...this.base.slice(0, gbuffer),
      ...(this.grassPass ? [this.grassPass] : []),
      ...this.base.slice(gbuffer, sun),
      ...this.shadows,
      ...(this.lightShadowPass ? [this.lightShadowPass] : []),
      ...this.base.slice(sun, sun + 1),
      ...(this.lightsPass ? [this.lightsPass] : []),
      ...this.base.slice(sun + 1, combine),
      ...(this.ambientOcclusion ? [this.ambientOcclusion] : []),
      ...this.base.slice(combine, forward),
      ...(this.temporal instanceof FsrPass ? [this.temporal.opaque] : []),
      ...this.base.slice(forward, overlay),
      ...(this.temporal ? [this.temporal] : []),
      ...(this.spatial ? [...smoothing, this.spatial] : []),
      ...(this.sharpen ? [this.sharpen] : []),
      ...this.base.slice(overlay),
      ...(this.spatial ? [] : smoothing),
      this.present,
    ];
    this.passNames = this.passes.map((pass: IRendererPass) => pass.name);
  }
}
