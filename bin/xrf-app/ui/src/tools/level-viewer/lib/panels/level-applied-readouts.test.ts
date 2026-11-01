import { describe, expect, it } from "@jest/globals";
import {
  ERenderAntialiasing,
  ERenderBloomMode,
  RenderAppliedReport,
  RenderViewOptions,
} from "@xrf/ipc/types/xrf-renderer";

import { mockRenderAppliedReport } from "@/core/render/fixtures/render.mocks";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/choice/render-preset";
import {
  ILevelAppliedReadout,
  ILevelAppliedSection,
  LEVEL_APPLIED_SECTIONS,
} from "@/tools/level-viewer/lib/panels/level-applied-readouts";

const FEATURES: IRenderFeatureSettings = {
  ...RENDER_PRESETS[ERenderPreset.BASE],
  antialiasing: ERenderAntialiasing.DLSS,
  enhancedBloom: { ...RENDER_PRESETS[ERenderPreset.BASE].enhancedBloom, mode: ERenderBloomMode.ENHANCED },
};

/** What a view asked, as far as the readouts compare against it. */
const REQUESTED: RenderViewOptions = {
  features: FEATURES,
  output: { upscaling: FEATURES.upscaling },
} as unknown as RenderViewOptions;

/**
 * @param label - A readout's label.
 * @param applied - What the renderer reported.
 * @returns What the line reads for it.
 */
function readLine(label: string, applied: RenderAppliedReport): string | null {
  const readout: ILevelAppliedReadout | undefined = LEVEL_APPLIED_SECTIONS.flatMap(
    (section: ILevelAppliedSection) => section.readouts
  ).find((it: ILevelAppliedReadout) => it.label === label);

  return readout ? readout.read(applied, REQUESTED) : null;
}

describe("LEVEL_APPLIED_SECTIONS", () => {
  it("says what was asked wherever the renderer drew something else, and nothing more where it drew the same", () => {
    expect(readLine("Antialiasing", mockRenderAppliedReport({ antialiasing: ERenderAntialiasing.DLSS }))).toBe("DLSS");
    expect(readLine("Antialiasing", mockRenderAppliedReport({ antialiasing: ERenderAntialiasing.FSR }))).toBe(
      "FSR 3.1 · asked DLSS"
    );
    expect(readLine("Bloom", mockRenderAppliedReport({ bloom: ERenderBloomMode.ENGINE }))).toBe(
      "Engine · asked Enhanced"
    );
  });

  it("names the sun shadows' cascades, size and filter against what was asked", () => {
    const line: string | null = readLine(
      "Sun shadows",
      mockRenderAppliedReport({
        shadows: { cascades: [20, 40], filter: FEATURES.shadows.filter, resolution: FEATURES.shadows.resolution },
      })
    );

    expect(line).toContain("20 / 40 m · asked");
    expect(line).toContain(`${FEATURES.shadows.resolution} texels`);
  });

  it("shows the environment only for a report that has one", () => {
    const environment: ILevelAppliedSection | undefined = LEVEL_APPLIED_SECTIONS.find(
      (section: ILevelAppliedSection) => section.title === "Applied environment"
    );

    expect(environment?.isShown?.(mockRenderAppliedReport({ environment: null }))).toBe(false);
  });
});
