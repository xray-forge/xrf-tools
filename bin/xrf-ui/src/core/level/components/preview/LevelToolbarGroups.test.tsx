import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import {
  ERenderAmbientOcclusionMethod,
  ERenderIndirectLightMode,
  RenderGraphSettings,
} from "@/core/ipc/types/xrf-renderer";
import { LevelCullingAction } from "@/core/level/components/preview/LevelCullingAction";
import { LevelOcclusionAction } from "@/core/level/components/preview/LevelOcclusionAction";
import { LevelOverlaysAction } from "@/core/level/components/preview/LevelOverlaysAction";
import { LevelShadingAction } from "@/core/level/components/preview/LevelShadingAction";
import { LevelSpawnAction } from "@/core/level/components/preview/LevelSpawnAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { DEFAULT_LEVEL_LOD_OPTIONS } from "@/core/level/lib/lod/level-lod-options";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import {
  DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import {
  TRenderAmbientOcclusionSettings,
  TRenderIndirectLightSettings,
} from "@/core/render/lib/settings/render-feature-settings";
import {
  DEFAULT_RENDER_GRAPH_SETTINGS,
  SERIAL_RENDER_GRAPH_SETTINGS,
} from "@/core/render/lib/settings/render-graph-settings";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

type TToggle = (option: keyof ILevelViewOptions) => void;

describe("level toolbar groups", () => {
  it("opens the shading on a click, and turns each surface switch over by its own checkbox", async () => {
    const onToggle = jest.fn<TToggle>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelShadingAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        shading={ELevelShading.FINAL}
        onToggle={onToggle}
        onChangeShading={() => {}}
      />
    );

    expect(getByRole("button", { name: "Shading" })).toHaveAccessibleDescription("Final frame");

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });
    await userEvent.click(getByRole("checkbox", { name: "Wireframe" }));
    await userEvent.click(getByRole("checkbox", { name: "Bumps" }));
    await userEvent.click(getByRole("checkbox", { name: "Wall marks" }));

    expect(onToggle.mock.calls).toEqual([["isWireframe"], ["isBumped"], ["isWallmarked"]]);
  });

  // Clay and the shader colours replace the textures checkbox: what the surfaces show is one choice with the targets.
  it("shows the frame as clay, by shader, or as one of its targets", async () => {
    const onChangeShading = jest.fn<(shading: ELevelShading) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelShadingAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        shading={ELevelShading.CLAY}
        onToggle={() => {}}
        onChangeShading={onChangeShading}
      />
    );

    expect(getByRole("button", { name: "Shading" })).toHaveAccessibleDescription("Clay");

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });
    await userEvent.click(getByRole("option", { name: "Depth" }));

    expect(onChangeShading).toHaveBeenCalledWith(ELevelShading.DEPTH);
  });

  it("names the spawned categories shown, and turns each over by its own checkbox", async () => {
    const onToggle = jest.fn<TToggle>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelSpawnAction
        options={{ ...DEFAULT_LEVEL_VIEW_OPTIONS, isSpawnedItems: false, isSpawnedWeapons: false }}
        onToggle={onToggle}
      />
    );

    expect(getByRole("button", { name: "Spawn" })).toHaveAccessibleDescription("Spawned props, lamps");

    await userEvent.click(getByRole("button", { name: "Spawn" }));
    await findByRole("dialog", { name: "Spawn" });

    expect(getByRole("checkbox", { name: "Items" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Items" }));
    await userEvent.click(getByRole("checkbox", { name: "Lamps" }));

    expect(onToggle.mock.calls).toEqual([["isSpawnedItems"], ["isSpawnedLamps"]]);
  });

  it("says why a culling the settings keep off cannot be turned on", async () => {
    const { getByRole, findByRole } = renderWithProviders(
      <LevelCullingAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        isOcclusionAvailable={false}
        isImpostorsAvailable
        lod={DEFAULT_LEVEL_LOD_OPTIONS}
        onToggle={() => {}}
        onChangeLod={() => {}}
        graph={DEFAULT_RENDER_GRAPH_SETTINGS}
        onChangeGraph={() => {}}
      />
    );

    await userEvent.click(getByRole("button", { name: "Culling" }));
    await findByRole("dialog", { name: "Culling" });

    expect(getByRole("checkbox", { name: "Occlusion culling" })).toBeDisabled();
    expect(getByRole("checkbox", { name: "Occlusion culling" })).toHaveAccessibleDescription(
      "Off in Settings, under Rendering"
    );
    expect(getByRole("checkbox", { name: "Impostors" })).toBeChecked();
  });

  it("sets the screen occlusion back to the settings and the baked one back to whole", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const onChangeHemiStrength = jest.fn<(hemiStrength: number) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelOcclusionAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        state={{ isAvailable: true, value: DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS }}
        features={{ ...mockLevelFeatureOptions(), ambientOcclusion: { radius: 2 } }}
        hemiStrength={0.5}
        indirectLight={DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS}
        onToggle={() => {}}
        onChange={onChange}
        onChangeHemiStrength={onChangeHemiStrength}
      />
    );

    await userEvent.click(getByRole("button", { name: "Occlusion" }));
    await findByRole("dialog", { name: "Occlusion" });
    await userEvent.click(getByRole("button", { name: "Back to the settings" }));
    await userEvent.click(getByRole("button", { name: "Back to the whole occlusion" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), ambientOcclusion: {} });
    expect(onChangeHemiStrength).toHaveBeenLastCalledWith(1);
  });

  it("switches the view to VBAO, offering its strengths only then", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const vbao: TRenderAmbientOcclusionSettings = {
      ...DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
      method: ERenderAmbientOcclusionMethod.VBAO,
    };

    function render(value: TRenderAmbientOcclusionSettings): ReactElement {
      return (
        <LevelOcclusionAction
          options={DEFAULT_LEVEL_VIEW_OPTIONS}
          state={{ isAvailable: true, value }}
          features={mockLevelFeatureOptions()}
          hemiStrength={1}
          indirectLight={DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS}
          onToggle={() => {}}
          onChange={onChange}
          onChangeHemiStrength={() => {}}
        />
      );
    }

    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(
      render(DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS)
    );

    await userEvent.click(getByRole("button", { name: "Occlusion" }));
    await findByRole("dialog", { name: "Occlusion" });

    expect(queryByRole("slider", { name: "Thickness" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "VBAO" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      ambientOcclusion: { method: ERenderAmbientOcclusionMethod.VBAO },
    });

    rerender(render(vbao));

    expect(getByRole("slider", { name: "Thickness" })).toHaveAttribute("aria-valuetext", "0.25 m");
    expect(getByRole("slider", { name: "Accumulation" })).toHaveAttribute("aria-valuetext", "8 frames");
    expect(getByRole("button", { hidden: true, name: "Occlusion" })).toHaveAccessibleDescription(
      "VBAO occlusion over 1.00 m, high quality, baked at 100%"
    );
  });

  it("turns the view's indirect light to enhanced, offering its intensity only then, and back to the settings", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const enhanced: TRenderIndirectLightSettings = {
      ...DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
      mode: ERenderIndirectLightMode.ENHANCED,
    };

    function render(indirectLight: TRenderIndirectLightSettings): ReactElement {
      return (
        <LevelOcclusionAction
          options={DEFAULT_LEVEL_VIEW_OPTIONS}
          state={{ isAvailable: true, value: DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS }}
          features={mockLevelFeatureOptions()}
          hemiStrength={1}
          indirectLight={indirectLight}
          onToggle={() => {}}
          onChange={onChange}
          onChangeHemiStrength={() => {}}
        />
      );
    }

    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(
      render(DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS)
    );

    await userEvent.click(getByRole("button", { name: "Occlusion" }));
    await findByRole("dialog", { name: "Occlusion" });

    expect(getByRole("checkbox", { name: "Indirect light" })).not.toBeChecked();
    expect(queryByRole("slider", { name: "Intensity" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Indirect light" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      indirectLight: { mode: ERenderIndirectLightMode.ENHANCED },
    });

    rerender(render(enhanced));

    expect(getByRole("checkbox", { name: "Indirect light" })).toBeChecked();
    expect(getByRole("slider", { name: "Intensity" })).toHaveAttribute("aria-valuetext", "100%");
    expect(getByRole("button", { hidden: true, name: "Occlusion" })).toHaveAccessibleDescription(
      "GTAO occlusion over 1.00 m, high quality, indirect light, baked at 100%"
    );

    await userEvent.click(getByRole("button", { name: "Back to the settings for the indirect light" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), indirectLight: {} });
  });

  it("names the overlays shown, and times the passes from the readouts", async () => {
    const onChangeGpuTimed = jest.fn<(isGpuTimed: boolean) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelOverlaysAction
        options={{ ...DEFAULT_LEVEL_VIEW_OPTIONS, isAxesVisible: true }}
        isGpuTimed={false}
        onToggle={() => {}}
        onChangeGpuTimed={onChangeGpuTimed}
      />
    );

    expect(getByRole("button", { name: "Overlays" })).toHaveAccessibleDescription("Showing axes, readouts");

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Grid" })).not.toBeChecked();
  });

  it("turns the frame graph's optimizations off one at a time or all at once from the culling, to bisect a capture", async () => {
    const onChangeGraph = jest.fn<(graph: RenderGraphSettings) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelCullingAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        isOcclusionAvailable
        isImpostorsAvailable
        lod={DEFAULT_LEVEL_LOD_OPTIONS}
        onToggle={() => {}}
        onChangeLod={() => {}}
        graph={DEFAULT_RENDER_GRAPH_SETTINGS}
        onChangeGraph={onChangeGraph}
      />
    );

    await userEvent.click(getByRole("button", { name: "Culling" }));
    await findByRole("dialog", { name: "Culling" });
    await userEvent.click(getByRole("checkbox", { name: "Merge render passes" }));

    expect(onChangeGraph).toHaveBeenLastCalledWith({ ...DEFAULT_RENDER_GRAPH_SETTINGS, isMerging: false });

    await userEvent.click(getByRole("checkbox", { name: "Frame graph" }));

    expect(onChangeGraph).toHaveBeenLastCalledWith(SERIAL_RENDER_GRAPH_SETTINGS);
  });

  // The marker says where the level's sunlight comes from, which the weather's own sun in the sky already shows by day.
  it("marks the sun among the overlays, off unless asked for", async () => {
    const onToggle = jest.fn<TToggle>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelOverlaysAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        isGpuTimed={false}
        onToggle={onToggle}
        onChangeGpuTimed={() => {}}
      />
    );

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Sun" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Sun" }));

    expect(onToggle).toHaveBeenCalledWith("isSunMarked");
  });
});
