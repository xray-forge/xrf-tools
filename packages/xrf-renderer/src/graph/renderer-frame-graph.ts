import { Nullable } from "@xrf/types";
import { PerspectiveCamera, RenderTarget, Texture, WebGPURenderer } from "three/webgpu";

import { ERendererAmbientOcclusionQuality } from "#/contract/renderer-ambient-occlusion-quality";
import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { IRendererFeatureSettings } from "#/contract/renderer-feature-settings";
import { TRendererSmoothingAntialiasing } from "#/contract/renderer-smoothing-antialiasing";
import { TRendererTemporalAntialiasing } from "#/contract/renderer-temporal-antialiasing";
import { ERendererPass } from "#/contract/scene/renderer-pass";
import { createBaseFramePasses, IBaseFramePasses } from "#/graph/base-frame-passes";
import { ICompilingFrame } from "#/graph/compiling-frame";
import { IFrameCompileTargets } from "#/graph/frame-compile-targets";
import { IFrameOptionalPasses } from "#/graph/frame-optional-passes";
import { toFramePassOrder } from "#/graph/frame-pass-order";
import { IFramePlan, toFramePlan } from "#/graph/frame-plan";
import { IFramePlanShadows } from "#/graph/frame-plan-shadows";
import { FrameStage } from "#/graph/frame-stage";
import { createOcclusionFramePasses, IOcclusionFramePasses } from "#/graph/occlusion-frame-passes";
import { AmbientOcclusionPass } from "#/pass/ambient-occlusion-pass";
import { AntialiasPass } from "#/pass/antialias/antialias-pass";
import { ExposurePass } from "#/pass/exposure-pass";
import { FsrPass } from "#/pass/fsr/fsr-pass";
import { GrassPass } from "#/pass/grass-pass";
import { LightShadowPass } from "#/pass/light-shadow-pass";
import { LightsPass } from "#/pass/lights-pass";
import { MotionBackgroundPass } from "#/pass/motion-background-pass";
import { PresentPass } from "#/pass/present-pass";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererScenePass, isRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { ShadowPass } from "#/pass/shadow-pass";
import { SharpenPass } from "#/pass/sharpen-pass";
import { SpatialUpscalePass } from "#/pass/spatial-upscale-pass";
import { TemporalAntialiasPass } from "#/pass/temporal-antialias-pass";
import { TemporalJitter } from "#/pass/temporal-jitter";
import { ITemporalUpscaler } from "#/pass/temporal-upscaler";
import { WaterDistortionPass } from "#/pass/water-distortion-pass";
import { WaterPass } from "#/pass/water-pass";
import { IRendererFrameJitter } from "#/sampling/renderer-frame-jitter";
import { IRendererFrameSize, isSameRendererFrameSize, toRendererFrameSize } from "#/sampling/renderer-frame-size";
import { SceneGrass } from "#/scene/grass/scene-grass";
import { SceneLights } from "#/scene/lights/scene-lights";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { RendererPassInspector } from "#/timing/renderer-pass-inspector";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** What the frame is drawn by and at: sized once each, a pass joining later sized as it joins. */
interface IFrameSizing {
  readonly renderer: WebGPURenderer;
  readonly size: IRendererFrameSize;
}

/** What RCAS is made for: the upscaled frame it sharpens, and whether it denoises too. */
interface IFrameSharpening {
  readonly isDenoised: boolean;
  readonly upscaled: RenderTarget;
}

/**
 * The frame: every target it draws into, and its passes in order, the picture presented last. The features make a
 * plan; each optional stage is made for its part of it and kept while that stands, a stage reading another's output
 * made again with it. A stage drawing a consumer scene joins once the compile lane admits it: until then the frame
 * draws as if it were off, and so does whatever reads it.
 */
export class RendererFrameGraph implements ICompilingFrame {
  public readonly targets: RendererTargets = new RendererTargets();
  /** The last pass, which a capture also draws into a target of its own. */
  public readonly present: PresentPass;
  /** Every pass's name, in frame order, as the frame report states them. */
  public passNames: ReadonlyArray<string> = [];

  private readonly uniforms: RendererUniforms;
  private readonly cull: StaticCull;
  private readonly casters: IStaticShadowCasters;
  private readonly grass: SceneGrass;
  private readonly lights: SceneLights;
  private readonly base: IBaseFramePasses;
  private readonly stages = {
    ambientOcclusion: new FrameStage<AmbientOcclusionPass>(release),
    distortion: new FrameStage<WaterDistortionPass>(release),
    exposure: new FrameStage<ExposurePass>(release),
    grass: new FrameStage<GrassPass>(release),
    jitter: new FrameStage<TemporalJitter>((jitter: TemporalJitter) => jitter.dispose()),
    lightShadows: new FrameStage<LightShadowPass>(release),
    lights: new FrameStage<LightsPass>(release),
    motionBackground: new FrameStage<MotionBackgroundPass>(release),
    occlusion: new FrameStage<IOcclusionFramePasses>((passes: IOcclusionFramePasses) =>
      Object.values(passes).forEach(release)
    ),
    resolve: new FrameStage<ITemporalUpscaler>(release),
    shadows: new FrameStage<ReadonlyArray<ShadowPass>>((passes: ReadonlyArray<ShadowPass>) => passes.forEach(release)),
    sharpen: new FrameStage<SharpenPass>(release),
    smoothing: new FrameStage<AntialiasPass>(release),
    spatial: new FrameStage<SpatialUpscalePass>(release),
    water: new FrameStage<WaterPass>(release),
  };

  /** The passes drawn, in frame order. */
  private passes: ReadonlyArray<IRendererPass> = [];
  /** Every pass the stages hold, in frame order: the drawn ones, and those held back until what they read joins. */
  private held: ReadonlyArray<IRendererPass> = [];
  /** The passes among them drawing the consumer's scenes, in frame order: the drawn ones and the joining ones. */
  private scenePasses: ReadonlyArray<IRendererScenePass> = [];
  /** The scene passes waiting to join, drawn once the compile lane admits them. */
  private joining: ReadonlyArray<IRendererScenePass> = [];
  /** The scene passes compiled for everything the scene draws: every base one, and each stage's once admitted. */
  private readonly admitted: WeakSet<IRendererScenePass> = new WeakSet();
  private plan: Nullable<IFramePlan> = null;
  private renderer: Nullable<WebGPURenderer> = null;
  /** The output's size, in device pixels, which the plan's upscale divides. */
  private width: number = 1;
  private height: number = 1;
  private sizing: Nullable<IFrameSizing> = null;
  /** Whether a configure sized the frame again since a frame last asked, which that frame reports as its own resize. */
  private isResizedByConfigure: boolean = false;
  /** What each pass and the targets were last sized by, so each is sized once for it. */
  private readonly sized: WeakMap<object, IFrameSizing> = new WeakMap();

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
    this.cull = cull;
    this.casters = casters;
    this.grass = grass;
    this.lights = lights;
    this.present = new PresentPass(this.targets, uniforms.camera);
    this.base = createBaseFramePasses(this.targets, uniforms, overlays, cull);
    // In the frame from the first, so everything the scene ever draws is compiled for them.
    Object.values(this.base)
      .filter(isRendererScenePass)
      .forEach((pass: IRendererScenePass) => this.admitted.add(pass));
    // The water's shaders read the frame's depth behind it, a texture that stays while its memory comes and goes.
    uniforms.water.depth.value = this.targets.waterDepth;
    this.link();
  }

  /** The frame's size as last sized, the scene's as drawn among it. */
  public get size(): IRendererFrameSize {
    return this.sizing?.size ?? toRendererFrameSize(this.width, this.height, 1);
  }

  /**
   * Where what the frame draws now compiles: its passes drawing the consumer's scenes and those joining to, the shadows
   * and the grass.
   */
  public get compileTargets(): IFrameCompileTargets {
    return {
      grass: this.targets.gbuffer,
      joining: this.joining,
      passes: this.scenePasses,
      shadow: { camera: this.uniforms.shadows.cascades[0].camera, target: this.targets.shadows[0] },
    };
  }

  /** The consumer's scenes the frame draws or is joining to draw, whose materials an object waits for. */
  public get framePasses(): ReadonlySet<ERendererPass> {
    return new Set(this.scenePasses.map((pass: IRendererScenePass) => pass.scene));
  }

  /** Whether a stage waits to join, which a frame drawn now is drawn without. */
  public get isJoining(): boolean {
    return this.joining.length > 0;
  }

  /** Where this frame's samples stand within the pixel, while a resolve jitters them. */
  public get jitter(): Nullable<IRendererFrameJitter> {
    return this.stages.jitter.value?.state ?? null;
  }

  /**
   * Puts into the frame the stages the features want and takes out the ones they do not, whose targets go with them.
   *
   * @param features - What the features are set to.
   */
  public configure(features: IRendererFeatureSettings): void {
    const plan: IFramePlan = toFramePlan(features);
    const { stages, targets, uniforms, casters, cull } = this;
    const isResolved: Nullable<true> = toWanted(plan.resolve !== null);

    this.plan = plan;
    stages.jitter.reconcile(isResolved, () => new TemporalJitter(uniforms.motion));
    stages.motionBackground.reconcile(isResolved, () => new MotionBackgroundPass(targets, uniforms.motion));
    targets.setWatered(plan.isWatered);
    stages.water.reconcile(toWanted(plan.isWatered), () => new WaterPass(targets, uniforms));
    stages.distortion.reconcile(toWanted(plan.isDistorted), () => new WaterDistortionPass(targets, uniforms));
    stages.grass.reconcile(toWanted(plan.isGrassy), () => new GrassPass(this.grass, targets));
    stages.exposure.reconcile(toWanted(plan.isExposed), () => new ExposurePass(targets, uniforms.exposure));
    stages.exposure.value?.setSettings(features.exposure);
    stages.occlusion.reconcile(toWanted(plan.isOccluding), () => createOcclusionFramePasses(targets, cull));
    cull.setOccluding(plan.isOccluding);
    stages.lights.reconcile(toWanted(plan.isLit), () => new LightsPass(this.lights, targets, uniforms));
    stages.lightShadows.reconcile(
      toWanted(plan.isLightShadowed),
      () => new LightShadowPass(this.lights.shadows, targets, casters, cull)
    );
    stages.ambientOcclusion.reconcile(
      plan.ambientOcclusion,
      (quality: ERendererAmbientOcclusionQuality) => new AmbientOcclusionPass(quality, targets, uniforms.camera)
    );
    stages.shadows.reconcile(
      plan.shadows,
      ({ count, resolution }: IFramePlanShadows) =>
        Array.from(
          { length: count },
          (_: unknown, view: number) =>
            new ShadowPass(view, targets, casters, cull, uniforms.shadows, uniforms.treeWind, resolution)
        ),
      ({ count, resolution }: IFramePlanShadows) => `${count}:${resolution}`
    );
    stages.resolve.reconcile(plan.resolve, (mode: TRendererTemporalAntialiasing) =>
      mode === ERendererAntialiasing.FSR2
        ? new FsrPass(targets, uniforms)
        : new TemporalAntialiasPass(targets, uniforms)
    );
    stages.smoothing.reconcile(
      plan.smoothing,
      (mode: TRendererSmoothingAntialiasing) => new AntialiasPass(mode, targets)
    );
    // FSR 1 reads what the smoothing finished, so it is made again with it.
    stages.spatial.reconcile(
      plan.isSpatial ? stages.smoothing.generation : null,
      () => new SpatialUpscalePass(stages.smoothing.value?.output.texture ?? targets.scene.texture, targets)
    );

    const upscaled: Nullable<RenderTarget> = stages.resolve.value?.output ?? stages.spatial.value?.output ?? null;

    // RCAS reads what was upscaled, so it is made again with the upscaler.
    stages.sharpen.reconcile(
      plan.sharpen && upscaled ? { ...plan.sharpen, upscaled } : null,
      ({ isDenoised, upscaled }: IFrameSharpening) => new SharpenPass(upscaled, isDenoised),
      ({ isDenoised }: IFrameSharpening) => `${isDenoised}:${stages.resolve.generation}:${stages.spatial.generation}`
    );

    const occlusion: Nullable<Texture> = stages.ambientOcclusion.value?.output ?? null;
    const shown: Nullable<RenderTarget> = stages.sharpen.value?.output ?? upscaled;

    stages.lights.value?.setFilter(features.lights.shadowFilter);
    stages.sharpen.value?.setSharpening(features.upscaling.sharpening);
    this.base.combine.setAmbientOcclusion(occlusion);
    this.present.setAmbientOcclusion(occlusion);
    this.base.overlay.setTarget(shown ?? targets.composite);
    this.present.setFrame(shown ?? stages.smoothing.value?.output ?? targets.scene);
    this.link();
    this.isResizedByConfigure ||= this.applySizing();
  }

  /**
   * @param renderer - The renderer the targets are drawn by.
   * @param width - Drawing buffer width, in device pixels: the output's.
   * @param height - Drawing buffer height, in device pixels.
   * @returns Whether the frame was sized again, which reallocated its targets.
   */
  public resize(renderer: WebGPURenderer, width: number, height: number): boolean {
    this.renderer = renderer;
    this.width = width;
    this.height = height;

    const isResized: boolean = this.applySizing() || this.isResizedByConfigure;

    this.isResizedByConfigure = false;

    return isResized;
  }

  /**
   * @param view - The view's camera, its matrices current.
   * @returns The camera the scene draws with this frame: the view's, offset within the pixel while a resolve jitters it.
   */
  public takeCamera(view: PerspectiveCamera): PerspectiveCamera {
    return this.stages.jitter.value?.take(view) ?? view;
  }

  public admit(pass: IRendererScenePass): void {
    if (this.joining.includes(pass)) {
      this.admitted.add(pass);
      this.link();
    }
  }

  /** Forgets what the resolve kept of the frames before, for a view that jumped. */
  public resetHistory(): void {
    this.stages.resolve.value?.resetHistory();
  }

  /**
   * @param frame - What the frame draws with.
   * @param inspector - What times each pass under its name.
   */
  public render(frame: IRendererFrame, inspector: RendererPassInspector): void {
    for (const pass of this.passes) {
      inspector.enter(pass.name);

      try {
        pass.render(frame);
      } finally {
        inspector.leave();
      }
    }

    this.stages.jitter.value?.advance();
  }

  public dispose(): void {
    Object.values(this.stages).forEach((stage: { dispose(): void }) => stage.dispose());
    Object.values(this.base).forEach(release);
    this.present.dispose();
    this.targets.dispose();
  }

  /**
   * Sizes the targets and every pass not yet sized for the frame as it is now, once there is a renderer to size by: a
   * new renderer, output size or upscale sizes all of them again, a pass joining the frame just itself.
   *
   * @returns Whether the frame's sizing changed, or its targets were allocated again.
   */
  private applySizing(): boolean {
    const { renderer, sizing: current, sized } = this;

    if (!renderer) {
      return false;
    }

    const size: IRendererFrameSize = toRendererFrameSize(this.width, this.height, this.plan?.upscale ?? 1);
    const sizing: IFrameSizing =
      current && current.renderer === renderer && isSameRendererFrameSize(current.size, size)
        ? current
        : { renderer, size };
    const jitter: Nullable<TemporalJitter> = this.stages.jitter.value;
    const isReallocated: boolean = this.targets.isStale;

    this.sizing = sizing;

    if (isReallocated || sized.get(this.targets) !== sizing) {
      this.targets.resize(renderer, sizing.size);
      sized.set(this.targets, sizing);
    }

    if (jitter && sized.get(jitter) !== sizing) {
      jitter.resize(sizing.size);
      sized.set(jitter, sizing);
    }

    // The held ones too: a pass joining is compiled into what it holds before it draws.
    for (const pass of this.held) {
      if (pass.resize && sized.get(pass) !== sizing) {
        pass.resize(renderer, sizing.size);
        sized.set(pass, sizing);
      }
    }

    return sizing !== current || isReallocated;
  }

  /** Orders the passes the stages hold now, a scene pass not admitted yet held back from the drawn ones. */
  private link(): void {
    const { stages, admitted } = this;
    const water: Nullable<WaterPass> = stages.water.value;
    const optional: IFrameOptionalPasses = {
      ambientOcclusion: stages.ambientOcclusion.value,
      exposure: stages.exposure.value,
      grass: stages.grass.value,
      lightShadows: stages.lightShadows.value,
      lights: stages.lights.value,
      motionBackground: stages.motionBackground.value,
      distortion: stages.distortion.value,
      occlusion: stages.occlusion.value,
      resolve: stages.resolve.value,
      sharpen: stages.sharpen.value,
      shadows: stages.shadows.value ?? [],
      smoothing: stages.smoothing.value,
      spatial: stages.spatial.value,
      water,
    };

    this.held = toFramePassOrder(this.base, optional, this.present);
    this.scenePasses = this.held.filter(isRendererScenePass);
    this.joining = this.scenePasses.filter((pass: IRendererScenePass) => !admitted.has(pass));
    this.passes = toFramePassOrder(
      this.base,
      { ...optional, water: water && admitted.has(water) ? water : null },
      this.present
    );
    this.passNames = this.passes.map((pass: IRendererPass) => pass.name);
  }
}

/**
 * @param isWanted - Whether a stage with no variants is wanted.
 * @returns What it is wanted for: nothing but that it is.
 */
function toWanted(isWanted: boolean): Nullable<true> {
  return isWanted ? true : null;
}

/**
 * @param pass - A pass leaving the frame.
 */
function release(pass: IRendererPass): void {
  pass.dispose();
}
