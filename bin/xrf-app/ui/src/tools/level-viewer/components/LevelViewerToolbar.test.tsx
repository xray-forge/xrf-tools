import { beforeEach, describe, expect, it } from "@jest/globals";

import { mockLevelViewContainer, renderLevelView, TLevelViewRenderResult } from "@/core/level/fixtures/level-container";
import { clearLocalStorage } from "@/core/storage";
import { LevelViewerToolbar } from "@/tools/level-viewer/components/LevelViewerToolbar";
import { LevelViewerCaptureService } from "@/tools/level-viewer/services/level-viewer-capture";
import { LevelViewerCaptureResultService } from "@/tools/level-viewer/services/level-viewer-capture-result";
import { LevelViewerScreenshotService } from "@/tools/level-viewer/services/level-viewer-screenshot";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelViewerToolbar", () => {
  it("lays every action out by what it changes: the view, the image, the lighting, the environment, the contents and the camera", () => {
    const { baseElement }: TLevelViewRenderResult = renderLevelView(
      <LevelViewerToolbar />,
      mockLevelViewContainer([
        LevelViewerCaptureService,
        LevelViewerCaptureResultService,
        LevelViewerScreenshotService,
        LevelViewerSnapshotService,
      ])
    );
    const actions: Array<string | null> = Array.from(baseElement.querySelectorAll("[data-testid$='-action']")).map(
      (element: Element) => element.getAttribute("data-testid")
    );

    expect(actions).toEqual([
      "level-shading-action",
      "level-overlays-action",
      "level-helpers-action",
      "level-look-action",
      "level-image-quality-action",
      "level-sun-action",
      "level-lights-action",
      "level-shadow-action",
      "level-occlusion-action",
      "level-reflections-action",
      "level-weather-action",
      "level-sky-action",
      "level-fog-action",
      "level-rain-action",
      "level-wind-action",
      "level-sound-action",
      "level-show-action",
      "level-particles-action",
      "level-grass-action",
      "level-water-action",
      "level-culling-action",
      "level-camera-action",
      "level-camera-effects-action",
      "level-go-to-action",
      "level-viewer-capture-action",
    ]);
  });
});
