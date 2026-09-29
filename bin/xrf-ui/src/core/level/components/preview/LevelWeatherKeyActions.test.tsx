import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelSkyAction } from "@/core/level/components/preview/LevelSkyAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_LEVEL_MANUAL_WEATHER, ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { mockLevelTextureReference } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

type TEdit = (patch: Partial<ILevelManualWeather>) => void;

async function open(getByRole: (role: string, options: { name: string }) => HTMLElement, label: string): Promise<void> {
  await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: label }) });
}

describe("level weather key actions", () => {
  it("edits a colour component as typed, once it is let go, leaving the others as held", async () => {
    const onEdit = jest.fn<TEdit>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelFogAction
        isOn
        manual={DEFAULT_LEVEL_MANUAL_WEATHER}
        isHazed={false}
        onToggle={() => {}}
        onEdit={onEdit}
        onHazed={() => {}}
      />
    );

    await open(getByRole, "Fog");
    await findByRole("dialog", { name: "Fog" });

    const green: HTMLElement = getByRole("textbox", { name: "fog_color green" });

    await userEvent.clear(green);
    await userEvent.type(green, "1.5");

    expect(onEdit).not.toHaveBeenCalled();

    await userEvent.type(green, "{Enter}");

    expect(onEdit).toHaveBeenCalledWith({ fogColor: [0.304609, 1.5, 0.367354] });
  });

  it("drops a component typed as no number", async () => {
    const onEdit = jest.fn<TEdit>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelFogAction
        isOn
        manual={DEFAULT_LEVEL_MANUAL_WEATHER}
        isHazed={false}
        onToggle={() => {}}
        onEdit={onEdit}
        onHazed={() => {}}
      />
    );

    await open(getByRole, "Fog");
    await findByRole("dialog", { name: "Fog" });

    const red: HTMLElement = getByRole("textbox", { name: "fog_color red" });

    await userEvent.clear(red);
    await userEvent.type(red, "-{Enter}");

    expect(onEdit).not.toHaveBeenCalled();
    expect(red).toHaveValue("0.304609");
  });

  it("fades the distance into the sky's haze on asking, the engine's sky by default", async () => {
    const onHazed = jest.fn<(isHazed: boolean) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelFogAction
        isOn
        manual={DEFAULT_LEVEL_MANUAL_WEATHER}
        isHazed={false}
        onToggle={() => {}}
        onEdit={() => {}}
        onHazed={onHazed}
      />
    );

    await open(getByRole, "Fog");
    await findByRole("dialog", { name: "Fog" });

    expect(getByRole("button", { name: "Sky (engine)" })).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(getByRole("button", { name: "Sky haze" }));

    expect(onHazed).toHaveBeenCalledWith(true);
  });

  it("stands the sun where the level was compiled against, and sets its keys back to default_clear's noon", async () => {
    const onEdit = jest.fn<TEdit>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelSunAction
        isOn
        manual={{ ...DEFAULT_LEVEL_MANUAL_WEATHER, sunAltitude: 10, sunLongitude: -80 }}
        sun={{ color: [1, 1, 1], direction: { x: 0, y: -1, z: 1 } }}
        onToggle={() => {}}
        onEdit={onEdit}
      />
    );

    await open(getByRole, "Sun");
    await findByRole("dialog", { name: "Sun" });

    expect(getByRole("dialog", { name: "Sun" }).textContent).toContain("80° up at a bearing of 10°");

    await userEvent.click(getByRole("button", { name: "Use the level's compiled sun" }));

    const [patch] = onEdit.mock.calls[0] ?? [];

    expect(patch?.sunAltitude).toBeCloseTo(0);
    expect(patch?.sunLongitude).toBeCloseTo(-45);

    await userEvent.click(getByRole("button", { name: "Back to default_clear's noon" }));

    expect(onEdit).toHaveBeenLastCalledWith({
      ambientColor: DEFAULT_LEVEL_MANUAL_WEATHER.ambientColor,
      hemisphereColor: DEFAULT_LEVEL_MANUAL_WEATHER.hemisphereColor,
      sunAltitude: DEFAULT_LEVEL_MANUAL_WEATHER.sunAltitude,
      sunColor: DEFAULT_LEVEL_MANUAL_WEATHER.sunColor,
      sunLongitude: DEFAULT_LEVEL_MANUAL_WEATHER.sunLongitude,
    });
  });

  it("offers the skies the game's weather names, with how many keyframes use each and which are missing", async () => {
    const onEdit = jest.fn<TEdit>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelSkyAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        manual={DEFAULT_LEVEL_MANUAL_WEATHER}
        skies={[
          { texture: mockLevelTextureReference("sky\\sky_night"), uses: 7 },
          { texture: mockLevelTextureReference("sky\\gone", false), uses: 1 },
        ]}
        clouds={[]}
        onToggle={() => {}}
        onEdit={onEdit}
      />
    );

    await userEvent.click(getByRole("button", { name: "Sky" }));
    await findByRole("dialog", { name: "Sky" });
    await userEvent.click(getByRole("combobox", { name: "sky_texture" }));

    expect(getByRole("listbox").textContent).toMatch(/sky\\sky_night7×.*sky\\gonemissing/);

    await userEvent.click(getByRole("option", { name: /sky_night/ }));

    expect(onEdit).toHaveBeenCalledWith({ skyTexture: "sky\\sky_night" });
  });

  it("takes a clouds reference typed in, and none for an empty one", async () => {
    const onEdit = jest.fn<TEdit>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelSkyAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        manual={DEFAULT_LEVEL_MANUAL_WEATHER}
        skies={[]}
        clouds={[]}
        onToggle={() => {}}
        onEdit={onEdit}
      />
    );

    await userEvent.click(getByRole("button", { name: "Sky" }));
    await findByRole("dialog", { name: "Sky" });

    const field: HTMLElement = getByRole("combobox", { name: "clouds_texture" });

    await userEvent.clear(field);
    await userEvent.type(field, "sky\\clouds_storm{Enter}");

    expect(onEdit).toHaveBeenLastCalledWith({ cloudsTexture: "sky\\clouds_storm" });
  });
});
