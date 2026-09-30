import { describe, expect, it, jest } from "@jest/globals";
import { PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";

import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { toRendererFeatureSettings } from "#/contract/renderer-feature-choice";
import { ERendererPass } from "#/contract/scene/renderer-pass";
import { DEFAULT_RENDER_FRAME_PACING } from "#/frame/render-frame-pacing";
import { IRendererFrame } from "#/pass/renderer-frame";
import { RendererTargets } from "#/pass/renderer-targets";
import { WallmarkPass } from "#/pass/wallmark-pass";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";

interface IFakeRenderer {
  setRenderTarget: jest.Mock;
  render: jest.Mock;
}

function toFrame(renderer: IFakeRenderer, scenes: TPassRecord<Scene>, isWallmarkDrawn: boolean): IRendererFrame {
  return {
    camera: new PerspectiveCamera(),
    jitter: null,
    renderer: renderer as unknown as WebGPURenderer,
    scenes,
    settings: {
      backdrop: null,
      debugView: ERendererDebugView.FINAL,
      features: toRendererFeatureSettings({}),
      hemiStrength: 1,
      isBumped: true,
      isGpuTimed: false,
      isLit: true,
      isSkyDrawn: false,
      isSkyHazed: false,
      isTextured: true,
      isWallmarkDrawn,
      isWireframe: false,
      pacing: DEFAULT_RENDER_FRAME_PACING,
      tonemapScale: 1,
    },
    time: 0,
    viewCamera: new PerspectiveCamera(),
  };
}

describe("WallmarkPass", () => {
  it("draws the wall mark scene into the albedo while the marks are drawn, and nothing while they are not", () => {
    const targets: RendererTargets = new RendererTargets();
    const pass: WallmarkPass = new WallmarkPass(targets);
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const renderer: IFakeRenderer = { render: jest.fn(), setRenderTarget: jest.fn() };

    pass.render(toFrame(renderer, scenes, true));

    expect(renderer.setRenderTarget).toHaveBeenCalledWith(targets.wallmarks);
    expect(renderer.render.mock.calls[0]?.[0]).toBe(scenes[ERendererPass.WALLMARK]);

    renderer.render.mockClear();
    renderer.setRenderTarget.mockClear();
    pass.render(toFrame(renderer, scenes, false));

    expect(renderer.setRenderTarget).not.toHaveBeenCalled();
    expect(renderer.render).not.toHaveBeenCalled();
  });
});
