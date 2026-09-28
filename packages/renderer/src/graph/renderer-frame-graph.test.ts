import { afterAll, beforeAll, describe, expect, it, jest } from "@jest/globals";
import { PerspectiveCamera, RenderTarget, Scene, WebGPURenderer } from "three/webgpu";

import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { IRendererFeatureSettings } from "#/contract/renderer-feature-settings";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { ERendererRenderScale } from "#/contract/renderer-render-scale";
import { DEFAULT_RENDER_FRAME_PACING } from "#/frame/render-frame-pacing";
import { RendererFrameGraph } from "#/graph/renderer-frame-graph";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { toPassRecord } from "#/scene/pass-record";
import { RendererScene } from "#/scene/renderer-scene";
import { RendererPassInspector } from "#/timing/renderer-pass-inspector";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

const BASE: IRendererFeatureSettings = RENDERER_PRESETS[ERendererPreset.BASE];

const PLAIN: IRendererFeatureSettings = {
  ...BASE,
  ambientOcclusion: { ...BASE.ambientOcclusion, isEnabled: false },
  antialiasing: ERendererAntialiasing.NONE,
  exposure: { ...BASE.exposure, isEnabled: false },
  grass: { ...BASE.grass, isEnabled: false },
  isOcclusionCulled: false,
  lights: { ...BASE.lights, isEnabled: false },
  shadows: { ...BASE.shadows, isEnabled: false },
  water: { ...BASE.water, isDistorted: false, isEnabled: false },
};

interface IFakeRenderer {
  renderer: WebGPURenderer;
  allocated: Array<RenderTarget>;
}

function createRenderer(): IFakeRenderer {
  const allocated: Array<RenderTarget> = [];
  const renderer = {
    _textures: { get: (): object => ({}) },
    initRenderTarget: (target: RenderTarget): void => {
      allocated.push(target);
    },
  };

  return { allocated, renderer: renderer as unknown as WebGPURenderer };
}

function createGraph(): RendererFrameGraph {
  const uniforms: RendererUniforms = new RendererUniforms();
  const scene: RendererScene = new RendererScene(uniforms, () => {});
  const overlays: RendererOverlays = new RendererOverlays(scene.skeletons, uniforms.lighting.sunDirection);

  return new RendererFrameGraph(uniforms, overlays, scene.staticCull, scene.shadowCasters, scene.grass, scene.lights);
}

function createSizedGraph(features: IRendererFeatureSettings): [RendererFrameGraph, IFakeRenderer] {
  const graph: RendererFrameGraph = createGraph();
  const fake: IFakeRenderer = createRenderer();

  graph.configure(features);
  graph.resize(fake.renderer, 64, 48);

  return [graph, fake];
}

describe("RendererFrameGraph", () => {
  // SMAA's lookups decode off the page, which node cannot, and say so.
  let error: jest.SpiedFunction<typeof console.error>;

  beforeAll(() => {
    error = jest.spyOn(console, "error").mockImplementation(() => {});
  });

  afterAll(() => error.mockRestore());

  it("puts the water in the frame while it is on, and its distortion only while that is on too", () => {
    const graph: RendererFrameGraph = createGraph();

    graph.configure(PLAIN);

    expect(graph.passNames).not.toContain("water");
    expect(graph.passNames).not.toContain("distortion");

    graph.configure({ ...PLAIN, water: { ...PLAIN.water, isEnabled: true } });

    expect(graph.passNames).toContain("water");
    expect(graph.passNames).not.toContain("distortion");

    graph.configure({ ...PLAIN, water: { ...PLAIN.water, isDistorted: true, isEnabled: true } });

    expect(graph.passNames.slice(graph.passNames.indexOf("water"))).toEqual([
      "water",
      "forward",
      "distortion",
      "overlay",
      "present",
    ]);
  });

  // An optional pass drawing a consumer scene compiles with the rest, or its first draw would compile on the frame.
  it("compiles the scene passes the frame holds now, the water's with them while it is on", () => {
    const graph: RendererFrameGraph = createGraph();

    function toNames(): Array<string> {
      return graph.compileTargets.passes.map((pass: IRendererScenePass) => pass.name);
    }

    graph.configure(PLAIN);

    expect(toNames()).toEqual(["gbuffer", "wallmarks", "forward"]);

    graph.configure({ ...PLAIN, water: { ...PLAIN.water, isEnabled: true } });

    expect(toNames()).toEqual(["gbuffer", "wallmarks", "water", "forward"]);
    expect(graph.compileTargets.grass).toBe(graph.targets.gbuffer);
    expect(graph.compileTargets.shadow.target).toBe(graph.targets.shadows[0]);
  });

  it("allocates the water's targets with the frame while it is on, and the frame again as it joins or leaves", () => {
    const [graph, fake]: [RendererFrameGraph, IFakeRenderer] = createSizedGraph(PLAIN);
    const targets: RendererTargets = graph.targets;

    expect(fake.allocated).not.toContain(targets.water);
    expect(graph.resize(fake.renderer, 64, 48)).toBe(false);

    fake.allocated.length = 0;
    graph.configure({ ...PLAIN, water: { ...PLAIN.water, isEnabled: true } });

    expect(fake.allocated).toEqual(expect.arrayContaining([targets.scene, targets.composite, targets.water]));
    // Reported by the next frame, which reads back what was just allocated.
    expect(graph.resize(fake.renderer, 64, 48)).toBe(true);
    expect(graph.resize(fake.renderer, 64, 48)).toBe(false);

    fake.allocated.length = 0;
    graph.configure(PLAIN);

    expect(fake.allocated).toEqual(expect.arrayContaining([targets.scene, targets.composite]));
    expect(fake.allocated).not.toContain(targets.water);
    expect(targets.isStale).toBe(false);
    expect(graph.resize(fake.renderer, 64, 48)).toBe(true);
  });

  it("reports a configure that sized the frame again as the next frame's resize, once", () => {
    const [graph, fake]: [RendererFrameGraph, IFakeRenderer] = createSizedGraph(PLAIN);

    expect(graph.resize(fake.renderer, 64, 48)).toBe(false);

    graph.configure({ ...PLAIN, upscaling: { ...PLAIN.upscaling, scale: ERendererRenderScale.PERFORMANCE } });

    expect(graph.size).toMatchObject({ renderHeight: 24, renderWidth: 32 });
    expect(graph.resize(fake.renderer, 64, 48)).toBe(true);
    expect(graph.resize(fake.renderer, 64, 48)).toBe(false);
    expect(graph.resize(fake.renderer, 80, 48)).toBe(true);
  });

  it("presents the frame combine finished, and draws the helpers over it, with no stage after it", () => {
    const [graph]: [RendererFrameGraph, IFakeRenderer] = createSizedGraph(PLAIN);

    expect(graph.present.frame).toBe(graph.targets.scene);
    expect(graph.passNames.slice(-2)).toEqual(["overlay", "present"]);
  });

  it("presents what the smoothing finished over the helpers, which draw over the composited frame first", () => {
    const [graph]: [RendererFrameGraph, IFakeRenderer] = createSizedGraph({
      ...PLAIN,
      antialiasing: ERendererAntialiasing.SMAA,
    });

    expect(graph.present.frame).not.toBe(graph.targets.scene);
    expect(graph.passNames.slice(-3)).toEqual(["overlay", "antialias", "present"]);
  });

  it("draws the helpers over what a resolve upscaled, and presents it", () => {
    const [graph]: [RendererFrameGraph, IFakeRenderer] = createSizedGraph({
      ...PLAIN,
      antialiasing: ERendererAntialiasing.TAA,
      upscaling: { scale: ERendererRenderScale.QUALITY, sharpening: 0 },
    });

    expect(graph.present.frame).not.toBe(graph.targets.scene);
    expect(graph.passNames).toContain("taa");
    expect(graph.passNames).not.toContain("rcas");
  });

  it("makes the stages reading another's output again with it: FSR 1 with the smoothing, RCAS with the upscaler", () => {
    const upscaled: IRendererFeatureSettings = {
      ...PLAIN,
      antialiasing: ERendererAntialiasing.FXAA,
      upscaling: { scale: ERendererRenderScale.PERFORMANCE, sharpening: 0.5 },
    };
    const [graph]: [RendererFrameGraph, IFakeRenderer] = createSizedGraph(upscaled);
    const sharpened: RenderTarget = graph.present.frame;

    graph.configure({ ...upscaled });

    expect(graph.present.frame).toBe(sharpened);

    graph.configure({ ...upscaled, antialiasing: ERendererAntialiasing.SMAA });

    expect(graph.present.frame).not.toBe(sharpened);
    expect(graph.passNames).toEqual(expect.arrayContaining(["antialias", "fsr1", "rcas"]));
  });

  // Left open, the next frame's renders would be timed under the pass that threw.
  it("closes the timing of a pass that throws, and lets what it threw through", () => {
    const [graph]: [RendererFrameGraph, IFakeRenderer] = createSizedGraph(PLAIN);
    const inspector: RendererPassInspector = new RendererPassInspector();
    const leave: jest.SpiedFunction<() => void> = jest.spyOn(inspector, "leave");
    const frame: IRendererFrame = {
      camera: new PerspectiveCamera(),
      jitter: null,
      // Nothing a pass calls exists, so the first pass throws.
      renderer: {} as WebGPURenderer,
      scenes: toPassRecord(() => new Scene()),
      settings: {
        backdrop: null,
        debugView: ERendererDebugView.FINAL,
        features: PLAIN,
        hemiStrength: 1,
        isBumped: true,
        isLit: true,
        isSkyDrawn: false,
        isWireframe: false,
        pacing: DEFAULT_RENDER_FRAME_PACING,
        tonemapScale: 1,
      },
      time: 0,
      viewCamera: new PerspectiveCamera(),
    };

    expect(() => graph.render(frame, inspector)).toThrow(TypeError);
    expect(leave).toHaveBeenCalledTimes(1);
  });
});
