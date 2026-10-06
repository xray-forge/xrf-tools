import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import {
  ERenderDebandingMode,
  ERenderDebandingQuality,
  ERenderFogMode,
  ERenderSunShaftsQuality,
} from "@/core/ipc/types/xrf-renderer";
import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelRainAction } from "@/core/level/components/preview/LevelRainAction";
import { LevelSkyAction } from "@/core/level/components/preview/LevelSkyAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_LEVEL_MANUAL_WEATHER, ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
  ILevelSunShaftsOptions,
} from "@/core/level/lib/weather/level-sun-shafts-options";
import {
  DEFAULT_RENDER_DEBANDING_SETTINGS,
  DEFAULT_RENDER_FOG_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import { mockLevelFeatureOptions, mockLevelTextureReference } from "@/fixtures/mocks/level.mocks";
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
        value={DEFAULT_RENDER_FOG_SETTINGS}
        features={mockLevelFeatureOptions()}
        onChangeFeatures={() => {}}
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
        value={DEFAULT_RENDER_FOG_SETTINGS}
        features={mockLevelFeatureOptions()}
        onChangeFeatures={() => {}}
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
        value={DEFAULT_RENDER_FOG_SETTINGS}
        features={mockLevelFeatureOptions()}
        onChangeFeatures={() => {}}
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
        manual={{ ...DEFAULT_LEVEL_MANUAL_WEATHER, sunAltitude: 10, sunLongitude: -80 }}
        sun={{ color: [1, 1, 1], direction: { x: 0, y: -1, z: 1 } }}
        drawnSun={null}
        suns={[]}
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        sunShafts={DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS}
        onToggle={() => {}}
        onChangeSunShafts={() => {}}
        onEdit={onEdit}
      />
    );

    // No toggle sits on it, so a click opens its settings.
    await userEvent.click(getByRole("button", { name: "Sun" }));
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
      sun: DEFAULT_LEVEL_MANUAL_WEATHER.sun,
      sunShaftsIntensity: DEFAULT_LEVEL_MANUAL_WEATHER.sunShaftsIntensity,
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
        debanding={DEFAULT_RENDER_DEBANDING_SETTINGS}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChangeFeatures={() => {}}
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
        debanding={DEFAULT_RENDER_DEBANDING_SETTINGS}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChangeFeatures={() => {}}
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

  it("strikes with a collection the game has, or none, and switches the thunder apart from the rain", async () => {
    const onEdit = jest.fn<TEdit>();
    const onToggle = jest.fn<(option: string) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelRainAction
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        manual={{ ...DEFAULT_LEVEL_MANUAL_WEATHER, thunderboltCollection: "collection_stancia" }}
        collections={[
          { file: "environment\\thunderbolt_collections.ltx", name: "collection_default", thunderbolts: ["a", "b"] },
        ]}
        onToggle={onToggle}
        onEdit={onEdit}
      />
    );

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });
    await userEvent.click(getByRole("checkbox", { name: "Thunder" }));

    expect(onToggle).toHaveBeenCalledWith("isThundering");

    await userEvent.click(getByRole("combobox", { name: "thunderbolt_collection" }));

    // A collection the game no longer has is kept, so the keyframe still says what it names.
    expect(getByRole("listbox").textContent).toMatch(
      /None.*collection_stancianot in the game.*collection_default2 bolts/
    );

    await userEvent.click(getByRole("option", { name: /collection_default/ }));

    expect(onEdit).toHaveBeenCalledWith({ thunderboltCollection: "collection_default" });
  });

  // The sky draws the weather's sun or moon as the game does; the button says whether one shows now.
  it("says whether the sky draws a sun or a moon, naming it", () => {
    const props = {
      manual: DEFAULT_LEVEL_MANUAL_WEATHER,
      sun: null,
      suns: [],
      options: DEFAULT_LEVEL_VIEW_OPTIONS,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      onToggle: () => {},
      onChangeSunShafts: () => {},
      onEdit: () => {},
    };
    const { getByRole, rerender } = renderWithProviders(<LevelSunAction {...props} drawnSun={"moon_halo_full"} />);

    expect(getByRole("button", { name: "Sun" })).toHaveAccessibleDescription(/moon_halo_full in the sky$/);

    rerender(<LevelSunAction {...props} drawnSun={null} />);

    expect(getByRole("button", { name: "Sun" })).toHaveAccessibleDescription(/none in the sky$/);
  });

  // The lens flare and the shafts are the sun's: their switches and the keys a keyframe draws them by sit together.
  it("switches the lens flares and the sunshafts, and edits the sun and the shafts' density", async () => {
    const onEdit = jest.fn<TEdit>();
    const onToggle = jest.fn<(option: "isLensFlared" | "isSunShafted") => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelSunAction
        manual={DEFAULT_LEVEL_MANUAL_WEATHER}
        sun={null}
        drawnSun={"gradient1"}
        suns={["gradient1", "moon_halo_full"]}
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        sunShafts={DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS}
        onToggle={onToggle}
        onChangeSunShafts={() => {}}
        onEdit={onEdit}
      />
    );

    await userEvent.click(getByRole("button", { name: "Sun" }));
    await findByRole("dialog", { name: "Sun" });
    await userEvent.click(getByRole("checkbox", { name: "Lens flares" }));
    await userEvent.click(getByRole("checkbox", { name: "Sunshafts" }));

    expect(onToggle.mock.calls).toEqual([["isLensFlared"], ["isSunShafted"]]);

    await userEvent.click(getByRole("combobox", { name: "sun" }));
    await userEvent.click(await findByRole("option", { name: "moon_halo_full" }));

    expect(onEdit).toHaveBeenCalledWith({ sun: "moon_halo_full" });
  });

  it("sets the level's sun shafts' quality and the floor under their density", async () => {
    const onChangeSunShafts = jest.fn<(sunShafts: ILevelSunShaftsOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelSunAction
        manual={DEFAULT_LEVEL_MANUAL_WEATHER}
        sun={null}
        drawnSun={null}
        suns={[]}
        options={DEFAULT_LEVEL_VIEW_OPTIONS}
        sunShafts={DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS}
        onToggle={() => {}}
        onChangeSunShafts={onChangeSunShafts}
        onEdit={() => {}}
      />
    );

    await userEvent.click(getByRole("button", { name: "Sun" }));
    await findByRole("dialog", { name: "Sun" });
    await userEvent.click(getByRole("button", { name: "Low" }));

    expect(onChangeSunShafts).toHaveBeenCalledWith({ minimum: 0, quality: ERenderSunShaftsQuality.LOW });
    expect(getByRole("slider", { name: "Minimum" })).toHaveAttribute("aria-valuemax", "0.5");
  });

  it("draws the fog enhanced on asking, offering its strengths only then, and back to the settings", async () => {
    const onChangeFeatures = jest.fn<(features: ILevelFeatureOptions) => void>();

    function render(mode: ERenderFogMode): ReactElement {
      return (
        <LevelFogAction
          isOn
          manual={DEFAULT_LEVEL_MANUAL_WEATHER}
          isHazed={false}
          value={{ ...DEFAULT_RENDER_FOG_SETTINGS, mode }}
          features={mockLevelFeatureOptions()}
          onToggle={() => {}}
          onEdit={() => {}}
          onHazed={() => {}}
          onChangeFeatures={onChangeFeatures}
        />
      );
    }

    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(render(ERenderFogMode.ENGINE));

    await open(getByRole, "Fog");
    await findByRole("dialog", { name: "Fog" });

    expect(queryByRole("slider", { name: "Height" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(onChangeFeatures).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      fog: { mode: ERenderFogMode.ENHANCED },
    });

    rerender(render(ERenderFogMode.ENHANCED));

    expect(getByRole("slider", { name: "Height" })).toHaveAttribute("aria-valuetext", "8.0 m");
    expect(getByRole("slider", { name: "Density" })).toHaveAttribute("aria-valuetext", "+130%");
    expect(getByRole("slider", { name: "Scattering" })).toHaveAttribute("aria-valuetext", "70%");

    await userEvent.click(getByRole("button", { name: "Back to the settings for the fog" }));

    expect(onChangeFeatures).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), fog: {} });
  });

  it("debands the sky on asking, offering its quality and radius only then", async () => {
    const onChangeFeatures = jest.fn<(features: ILevelFeatureOptions) => void>();

    function render(mode: ERenderDebandingMode): ReactElement {
      return (
        <LevelSkyAction
          options={DEFAULT_LEVEL_VIEW_OPTIONS}
          manual={DEFAULT_LEVEL_MANUAL_WEATHER}
          skies={[]}
          clouds={[]}
          debanding={{ ...DEFAULT_RENDER_DEBANDING_SETTINGS, mode }}
          features={mockLevelFeatureOptions()}
          onToggle={() => {}}
          onEdit={() => {}}
          onChangeFeatures={onChangeFeatures}
        />
      );
    }

    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(render(ERenderDebandingMode.ENGINE));

    await userEvent.click(getByRole("button", { name: "Sky" }));
    await findByRole("dialog", { name: "Sky" });

    expect(queryByRole("slider", { name: "Radius" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Debanding" }));

    expect(onChangeFeatures).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      debanding: { mode: ERenderDebandingMode.ENHANCED },
    });

    rerender(render(ERenderDebandingMode.ENHANCED));

    expect(getByRole("slider", { name: "Radius" })).toHaveAttribute("aria-valuetext", "48 px");

    await userEvent.click(getByRole("button", { name: "Ultra" }));

    expect(onChangeFeatures).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      debanding: { quality: ERenderDebandingQuality.ULTRA },
    });
  });
});
