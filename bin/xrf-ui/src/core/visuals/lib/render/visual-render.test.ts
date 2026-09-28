import { describe, expect, it } from "@jest/globals";
import {
  DEFAULT_RENDERER_FEATURE_CHOICE,
  ERendererCameraController,
  ERendererDraw,
  IRendererFeatureSettings,
  IRendererGeometry,
  IRendererOrbitCamera,
  IRendererSettings,
  IRendererSurface,
  resolveRendererFeatures,
} from "@xrf/renderer";

import { toRawColor } from "@/core/render/lib/scene/render-color";
import {
  toVisualCamera,
  toVisualGeometry,
  toVisualObject,
  toVisualRendererSettings,
  toVisualSkeleton,
  toVisualSurface,
  VISUAL_RENDER_KEYS,
} from "@/core/visuals/lib/render/visual-render";
import { DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG } from "@/core/visuals/lib/scene/scene-config";
import { DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS } from "@/core/visuals/lib/scene/visual-view-options";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { IVisualSubmeshViews } from "@/core/visuals/lib/visual-views";
import { mockRenderSharedSettings } from "@/fixtures/mocks/render.mocks";
import { mockVisualModelViews, mockVisualSubmeshViews } from "@/fixtures/mocks/visual.mocks";

const FEATURES: IRendererFeatureSettings = resolveRendererFeatures(DEFAULT_RENDERER_FEATURE_CHOICE);

const TEXTURE: IVisualTextureFile = { bytes: new ArrayBuffer(8), isDecoded: false, logicalPath: "textures\\wall" };

function mockCutOut(): IVisualSubmeshViews {
  return mockVisualSubmeshViews({ surface: { alphaReference: 0.5, draw: ERendererDraw.CUT_OUT, isLit: true } });
}

describe("toVisualGeometry", () => {
  it("copies every array, so a put hands over the copy and the views keep their buffer", () => {
    const submesh: IVisualSubmeshViews = mockVisualSubmeshViews();
    const geometry: IRendererGeometry = toVisualGeometry(submesh);

    expect(geometry.position).not.toBe(submesh.positions);
    expect(geometry.index).not.toBe(submesh.indices);
    expect(geometry.groups).toEqual([{ count: 3, slot: 0, start: 0 }]);
  });
});

describe("toVisualSurface", () => {
  it("draws the base texture as the shader resolved it", () => {
    const surface: IRendererSurface = toVisualSurface(
      mockCutOut(),
      TEXTURE,
      null,
      DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS,
      DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
    );

    expect(surface).toMatchObject({ alphaReference: 0.5, color: undefined, draw: ERendererDraw.CUT_OUT, tiling: 1 });
    expect(surface.textures?.base).toBe("textures\\wall");
  });

  it("draws every surface solid while the alpha is off, for comparison", () => {
    const surface: IRendererSurface = toVisualSurface(
      mockCutOut(),
      TEXTURE,
      null,
      { ...DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, isAlphaVisible: false },
      DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
    );

    expect(surface.draw).toBe(ERendererDraw.OPAQUE);
    expect(surface.alphaReference).toBeUndefined();
  });

  it("lays the uv checker over a surface while the toolbar asks, tiled", () => {
    const surface: IRendererSurface = toVisualSurface(
      mockCutOut(),
      TEXTURE,
      null,
      { ...DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, isCheckerVisible: true },
      DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
    );

    expect(surface.textures?.base).toBe(VISUAL_RENDER_KEYS.checker);
    expect(surface.tiling).toBe(DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG.checkerRepeat);
  });

  it("paints an untextured surface the viewer's mesh colour rather than white", () => {
    const surface: IRendererSurface = toVisualSurface(
      mockCutOut(),
      null,
      null,
      DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS,
      DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
    );

    expect(surface.color).toEqual(toRawColor(DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG.meshColor));
  });
});

describe("toVisualObject", () => {
  it("skins a submesh only where the model has a skeleton to bind it to", () => {
    const skinned: IVisualSubmeshViews = mockVisualSubmeshViews({ skinIndices: new Uint16Array(4) });

    expect(toVisualObject(skinned, 0, true).skeleton).toBe(VISUAL_RENDER_KEYS.skeleton);
    expect(toVisualObject(skinned, 0, false).skeleton).toBeUndefined();
    expect(toVisualObject(mockVisualSubmeshViews(), 0, true).skeleton).toBeUndefined();
  });
});

describe("toVisualSkeleton", () => {
  it("copies the binds of a model with bones, and answers nothing for one without", () => {
    const binds: Float32Array = new Float32Array(12);

    expect(toVisualSkeleton(mockVisualModelViews({ skeletonBinds: binds }))?.binds).not.toBe(binds);
    expect(toVisualSkeleton(mockVisualModelViews())).toBeNull();
  });
});

describe("toVisualCamera", () => {
  it("frames the model's sphere from the viewer's direction, far enough to fit it", () => {
    const camera: IRendererOrbitCamera = toVisualCamera(
      { center: [1, 2, 3], radius: 2 },
      DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
    );
    const distance: number = Math.hypot(camera.position[0] - 1, camera.position[1] - 2, camera.position[2] - 3);

    expect(camera.kind).toBe(ERendererCameraController.ORBIT);
    expect(camera.target).toEqual([1, 2, 3]);
    expect(distance).toBeGreaterThan(2);
    expect(camera.near).toBeLessThan(distance);
  });
});

describe("toVisualRendererSettings", () => {
  it("draws against the viewer's backdrop with the exposure held at the engine's noon", () => {
    const settings: IRendererSettings = toVisualRendererSettings(
      { ...DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, isWireframe: true },
      DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG,
      mockRenderSharedSettings({ features: FEATURES })
    );

    expect(settings.backdrop).toBe(DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG.backgroundColor);
    expect(settings.isWireframe).toBe(true);
    expect(settings.isSkyDrawn).toBe(false);
    expect(settings.features.exposure.isEnabled).toBe(false);
  });
});
