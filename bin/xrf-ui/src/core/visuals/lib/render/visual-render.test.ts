import { describe, expect, it } from "@jest/globals";

import { ERenderCamera, ERenderOverlay, RenderCamera, RenderViewOptions } from "@/core/ipc/types/xrf-renderer";
import { toRawColor } from "@/core/render/lib/scene/render-color";
import { toVisualCamera, toVisualOverlays, toVisualViewOptions } from "@/core/visuals/lib/render/visual-render";
import { DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG } from "@/core/visuals/lib/scene/scene-config";
import { DEFAULT_VISUAL_LIGHTING } from "@/core/visuals/lib/scene/visual-lighting";
import { DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS } from "@/core/visuals/lib/scene/visual-view-options";
import { mockRenderFeatures } from "@/fixtures/mocks/render.mocks";

describe("toVisualCamera", () => {
  it("frames the model's sphere from the viewer's direction, far enough to fit it", () => {
    const camera: RenderCamera = toVisualCamera({ center: [1, 2, 3], radius: 2 }, DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG);
    const [x, y, z] = camera.position.map((it) => it ?? 0);
    const distance: number = Math.hypot(x - 1, y - 2, z - 3);

    expect(camera.kind).toBe(ERenderCamera.ORBIT);
    expect(camera.target).toEqual([1, 2, 3]);
    expect(distance).toBeGreaterThan(2);
    expect(camera.near ?? 0).toBeLessThan(distance);
  });
});

describe("toVisualViewOptions", () => {
  it("draws against the viewer's backdrop and light, with the exposure held at the engine's noon", () => {
    const options: RenderViewOptions = toVisualViewOptions(
      { ...DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, isWireframe: true },
      DEFAULT_VISUAL_LIGHTING,
      DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG,
      mockRenderFeatures()
    );

    expect(options.asset.backdrop).toEqual(toRawColor(DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG.backgroundColor));
    expect(options.asset.plainColor).toEqual(toRawColor(DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG.meshColor));
    expect(options.asset.lighting?.sunElevation).toBe(DEFAULT_VISUAL_LIGHTING.sunElevation);
    expect(options.mode.isWireframe).toBe(true);
    expect(options.show.isSkyVisible).toBe(false);
    expect(options.show.isFogged).toBe(false);
    expect(options.features.grass.isEnabled).toBe(false);
    expect(options.features.water.isEnabled).toBe(false);
    expect(options.features.exposure.isEnabled).toBe(false);
    // The one model stands as a prop, which is a group the view keeps drawn.
    expect(options.world.isSpawnedProps).toBe(true);
  });

  it("lays the uv checker over every surface while the toolbar asks, and draws them solid with the alpha off", () => {
    const options: RenderViewOptions = toVisualViewOptions(
      { ...DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, isAlphaVisible: false, isCheckerVisible: true },
      DEFAULT_VISUAL_LIGHTING,
      DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG,
      mockRenderFeatures()
    );

    expect(options.asset.checker).toBe(DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG.checkerRepeat);
    expect(options.show.isAlphaVisible).toBe(false);
  });
});

describe("toVisualOverlays", () => {
  const config = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG;

  it("draws the grid and the axes as the toolbar asks", () => {
    const kinds = toVisualOverlays(
      { isAxesVisible: true, isGridVisible: true, isSkeletonVisible: false },
      1,
      null,
      config
    ).map((it) => it.kind);

    expect(kinds).toEqual([ERenderOverlay.LINES, ERenderOverlay.LINES]);
  });

  it("draws the skeleton, and the marked joint only beside it", () => {
    const hidden = toVisualOverlays(
      { isAxesVisible: false, isGridVisible: false, isSkeletonVisible: false },
      1,
      [0, 1, 0],
      config
    );
    const shown = toVisualOverlays(
      { isAxesVisible: false, isGridVisible: false, isSkeletonVisible: true },
      1,
      [0, 1, 0],
      config
    );

    expect(hidden).toEqual([]);
    expect(shown).toEqual([
      { color: toRawColor(config.skeletonColor), isDepthTested: false, kind: ERenderOverlay.SKELETON },
      {
        color: toRawColor(config.highlightColor),
        isDepthTested: false,
        kind: ERenderOverlay.POINTS,
        positions: [0, 1, 0],
        size: config.highlightSize,
      },
    ]);
  });
});
