import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelCullingAction } from "@/core/level/components/preview/LevelCullingAction";
import { LevelOcclusionAction } from "@/core/level/components/preview/LevelOcclusionAction";
import { LevelOverlaysAction } from "@/core/level/components/preview/LevelOverlaysAction";
import { LevelSpawnAction } from "@/core/level/components/preview/LevelSpawnAction";
import { LevelSurfacesAction } from "@/core/level/components/preview/LevelSurfacesAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { DEFAULT_LEVEL_LOD_OPTIONS } from "@/core/level/lib/lod/level-lod-options";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS } from "@/core/render/lib/contract/renderer-ambient-occlusion-settings";
import { ERendererDebugView } from "@/core/render/lib/contract/renderer-debug-view";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

type TToggle = (option: keyof ILevelViewOptions) => void;

describe("level toolbar groups", () => {
  it("opens the surfaces on a click, and turns each over by its own checkbox", async () => {
    const onToggle = jest.fn<TToggle>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelSurfacesAction options={DEFAULT_LEVEL_VIEW_OPTIONS} onToggle={onToggle} />
    );

    expect(getByRole("button", { name: "Surfaces" })).toHaveAccessibleDescription(
      "Solid, textured, bumped, with wall marks"
    );

    await userEvent.click(getByRole("button", { name: "Surfaces" }));
    await findByRole("dialog", { name: "Surfaces" });
    await userEvent.click(getByRole("checkbox", { name: "Wireframe" }));
    await userEvent.click(getByRole("checkbox", { name: "Bumps" }));
    await userEvent.click(getByRole("checkbox", { name: "Wall marks" }));

    expect(onToggle.mock.calls).toEqual([["isWireframe"], ["isBumped"], ["isWallmarked"]]);
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
        state={{ isAvailable: true, value: DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS }}
        features={{ ...mockLevelFeatureOptions(), ambientOcclusion: { radius: 2 } }}
        hemiStrength={0.5}
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

  it("names the overlays shown, and times the passes from the readouts", async () => {
    const onChangeGpuTimed = jest.fn<(isGpuTimed: boolean) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelOverlaysAction
        options={{ ...DEFAULT_LEVEL_VIEW_OPTIONS, isAxesVisible: true }}
        isGpuTimed={false}
        debugView={ERendererDebugView.FINAL}
        onToggle={() => {}}
        onChangeGpuTimed={onChangeGpuTimed}
        onChangeDebugView={() => {}}
      />
    );

    expect(getByRole("button", { name: "Overlays" })).toHaveAccessibleDescription("Showing axes, readouts");

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Grid" })).not.toBeChecked();
  });

  it("shows one of the frame's targets instead of the frame", async () => {
    const onChangeDebugView = jest.fn<(debugView: ERendererDebugView) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelOverlaysAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        isGpuTimed={false}
        debugView={ERendererDebugView.FINAL}
        onToggle={() => {}}
        onChangeGpuTimed={() => {}}
        onChangeDebugView={onChangeDebugView}
      />
    );

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });
    await userEvent.click(getByRole("option", { name: "Depth" }));

    expect(onChangeDebugView).toHaveBeenCalledWith(ERendererDebugView.DEPTH);
  });
});
