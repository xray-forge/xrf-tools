import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { LevelWeatherDescription, SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import {
  ERenderDebandingMode,
  ERenderDebandingQuality,
  ERenderFogMode,
  ERenderRainMode,
  ERenderSunShaftsQuality,
  ERenderVolumetricMode,
} from "@/core/ipc/types/xrf-renderer";
import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelRainAction } from "@/core/level/components/preview/LevelRainAction";
import { LevelSkyAction } from "@/core/level/components/preview/LevelSkyAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_LEVEL_MANUAL_WEATHER, ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  DEFAULT_RENDER_DEBANDING_SETTINGS,
  DEFAULT_RENDER_FOG_SETTINGS,
  DEFAULT_RENDER_RAIN_SETTINGS,
  DEFAULT_RENDER_VOLUMETRIC_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import { mockLevelTextureReference, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelApplied,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/** What a case opens: a level, the weather its game names, and keys of the keyframe set by hand already set. */
interface IWeatherSetup {
  level?: Partial<SelectedLevelDescription>;
  weather?: Partial<LevelWeatherDescription>;
  manual?: Partial<ILevelManualWeather>;
}

/**
 * @param action - The weather popover to draw.
 * @param setup - What the case opens; no level unless a level or its weather is asked for.
 * @returns The rendered action and the services behind it, once the level asked for is open and its weather read.
 */
async function renderAction(action: ReactElement, setup: IWeatherSetup = {}): Promise<TLevelViewRenderResult> {
  const isOpen: boolean = Boolean(setup.level || setup.weather);

  if (isOpen) {
    setMockInvokeResponses({
      ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription(setup.level)),
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription(setup.weather)),
    });
  }

  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  // Each feature the popovers set starts from its default, whatever the preset draws it with.
  descriptor.renderSettings.setRendererOverrides({
    debanding: DEFAULT_RENDER_DEBANDING_SETTINGS,
    fog: DEFAULT_RENDER_FOG_SETTINGS,
    rain: DEFAULT_RENDER_RAIN_SETTINGS,
    volumetric: DEFAULT_RENDER_VOLUMETRIC_SETTINGS,
  });

  const rendered: TLevelViewRenderResult = renderLevelView(action, descriptor);

  if (isOpen) {
    await waitFor(() => expect(descriptor.weather.description.isReady).toBe(true));
    await waitFor(() => expect(descriptor.weather.cycle.isLoading).toBe(false));
  }

  const manual: Partial<ILevelManualWeather> | undefined = setup.manual;

  if (manual) {
    act(() => descriptor.weather.editManual(manual));
  }

  return rendered;
}

async function open(getByRole: (role: string, options: { name: string }) => HTMLElement, label: string): Promise<void> {
  await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: label }) });
}

beforeEach(() => {
  window.localStorage.clear();
});

afterEach(() => {
  resetMockInvoke();
});

describe("level weather key actions", () => {
  it("edits a colour component as typed, once it is let go, leaving the others as held", async () => {
    const { getByRole, findByRole, weather } = await renderAction(<LevelFogAction />);

    await open(getByRole, "Fog");
    await findByRole("dialog", { name: "Fog" });

    const green: HTMLElement = getByRole("textbox", { name: "fog_color green" });

    await userEvent.clear(green);
    await userEvent.type(green, "1.5");

    expect(weather.manual).toBeNull();

    await userEvent.type(green, "{Enter}");

    expect(weather.manual?.fogColor).toEqual([0.304609, 1.5, 0.367354]);
  });

  it("drops a component typed as no number", async () => {
    const { getByRole, findByRole, weather } = await renderAction(<LevelFogAction />);

    await open(getByRole, "Fog");
    await findByRole("dialog", { name: "Fog" });

    const red: HTMLElement = getByRole("textbox", { name: "fog_color red" });

    await userEvent.clear(red);
    await userEvent.type(red, "-{Enter}");

    expect(weather.manual).toBeNull();
    expect(red).toHaveValue("0.304609");
  });

  it("fades the distance into the sky's haze on asking, the engine's sky while off", async () => {
    const { getByRole, findByRole, view } = await renderAction(<LevelFogAction />);

    act(() => view.setOption("isSkyHazed", false));
    await open(getByRole, "Fog");
    await findByRole("dialog", { name: "Fog" });

    expect(getByRole("button", { name: "Sky (engine)" })).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(getByRole("button", { name: "Sky haze" }));

    expect(view.options.isSkyHazed).toBe(true);
  });

  it("stands the sun where the level was compiled against, and sets its keys back to default_clear's noon", async () => {
    const { getByRole, findByRole, weather } = await renderAction(<LevelSunAction />, {
      level: { sun: { color: [1, 1, 1], direction: { x: 0, y: -1, z: 1 } } },
      manual: { sunAltitude: 10, sunLongitude: -80 },
    });

    // No toggle sits on it, so a click opens its settings.
    await userEvent.click(getByRole("button", { name: "Sun" }));
    await findByRole("dialog", { name: "Sun" });

    expect(getByRole("dialog", { name: "Sun" }).textContent).toContain("80° up at a bearing of 10°");

    await userEvent.click(getByRole("button", { name: "Use the level's compiled sun" }));

    expect(weather.manual?.sunAltitude).toBeCloseTo(0);
    expect(weather.manual?.sunLongitude).toBeCloseTo(-45);

    await userEvent.click(getByRole("button", { name: "Back to default_clear's noon" }));

    expect(weather.manual).toMatchObject({
      ambientColor: DEFAULT_LEVEL_MANUAL_WEATHER.ambientColor,
      hemisphereColor: DEFAULT_LEVEL_MANUAL_WEATHER.hemisphereColor,
      sun: DEFAULT_LEVEL_MANUAL_WEATHER.sun,
      sunAltitude: DEFAULT_LEVEL_MANUAL_WEATHER.sunAltitude,
      sunColor: DEFAULT_LEVEL_MANUAL_WEATHER.sunColor,
      sunLongitude: DEFAULT_LEVEL_MANUAL_WEATHER.sunLongitude,
      sunShaftsIntensity: DEFAULT_LEVEL_MANUAL_WEATHER.sunShaftsIntensity,
    });
  });

  it("offers the skies the game's weather names, with how many keyframes use each and which are missing", async () => {
    const { getByRole, findByRole, weather } = await renderAction(<LevelSkyAction />, {
      weather: {
        skies: [
          { texture: mockLevelTextureReference("sky\\sky_night"), uses: 7 },
          { texture: mockLevelTextureReference("sky\\gone", false), uses: 1 },
        ],
      },
    });

    await userEvent.click(getByRole("button", { name: "Sky" }));
    await findByRole("dialog", { name: "Sky" });
    await userEvent.click(getByRole("combobox", { name: "sky_texture" }));

    expect(getByRole("listbox").textContent).toMatch(/sky\\sky_night7×.*sky\\gonemissing/);

    await userEvent.click(getByRole("option", { name: /sky_night/ }));

    expect(weather.manual?.skyTexture).toBe("sky\\sky_night");
  });

  it("takes a clouds reference typed in, and none for an empty one", async () => {
    const { getByRole, findByRole, weather } = await renderAction(<LevelSkyAction />);

    await userEvent.click(getByRole("button", { name: "Sky" }));
    await findByRole("dialog", { name: "Sky" });

    const field: HTMLElement = getByRole("combobox", { name: "clouds_texture" });

    await userEvent.clear(field);
    await userEvent.type(field, "sky\\clouds_storm{Enter}");

    expect(weather.manual?.cloudsTexture).toBe("sky\\clouds_storm");
  });

  it("strikes with a collection the game has, or none, and switches the thunder apart from the rain", async () => {
    const { getByRole, findByRole, view, weather } = await renderAction(<LevelRainAction />, {
      manual: { thunderboltCollection: "collection_stancia" },
      weather: {
        thunderboltCollections: [
          { file: "environment\\thunderbolt_collections.ltx", name: "collection_default", thunderbolts: ["a", "b"] },
        ],
      },
    });

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });
    await userEvent.click(getByRole("checkbox", { name: "Thunder" }));

    expect(view.options.isThundering).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isThundering);

    await userEvent.click(getByRole("combobox", { name: "thunderbolt_collection" }));

    // A collection the game no longer has is kept, so the keyframe still says what it names.
    expect(getByRole("listbox").textContent).toMatch(
      /None.*collection_stancianot in the game.*collection_default2 bolts/
    );

    await userEvent.click(getByRole("option", { name: /collection_default/ }));

    expect(weather.manual?.thunderboltCollection).toBe("collection_default");
  });

  it("gathers puddles on the ground as the rain's third section, offering their strengths only then", async () => {
    const { getByRole, findByRole, queryByRole, view } = await renderAction(<LevelRainAction />);

    expect(getByRole("button", { name: "Rain" })).toHaveAccessibleDescription("No rain, no thunder, no puddles");

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });

    expect(queryByRole("slider", { name: "Puddles" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Puddles" }));

    expect(view.features.rain).toEqual({ mode: ERenderRainMode.ENHANCED });
    expect(getByRole("checkbox", { name: "Puddles" })).toBeChecked();
    expect(getByRole("slider", { name: "Puddles" })).toHaveAttribute("aria-valuetext", "50%");
    expect(getByRole("slider", { name: "Reflectivity" })).toHaveAttribute("aria-valuetext", "40%");
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "300 m");
    expect(getByRole("button", { hidden: true, name: "Rain" })).toHaveAccessibleDescription(
      "No rain, no thunder, puddles at 50% of the sites"
    );

    await userEvent.click(getByRole("button", { name: "Back to the settings for the puddles" }));

    expect(view.features.rain).toEqual({});
  });

  // The sky draws the weather's sun or moon as the game does; the button says whether one shows now.
  it("says whether the sky draws a sun or a moon, naming it", async () => {
    const { getByRole, render } = await renderAction(<LevelSunAction />);

    noteLevelApplied(render, { sun: "moon_halo_full" });

    expect(getByRole("button", { name: "Sun" })).toHaveAccessibleDescription(/moon_halo_full in the sky$/);

    noteLevelApplied(render, { sun: null });

    expect(getByRole("button", { name: "Sun" })).toHaveAccessibleDescription(/none in the sky$/);
  });

  // The lens flare and the shafts are the sun's: their switches and the keys a keyframe draws them by sit together.
  it("switches the lens flares and the sunshafts, and edits the sun and the shafts' density", async () => {
    const { getByRole, findByRole, view, weather } = await renderAction(<LevelSunAction />, {
      weather: { suns: ["gradient1", "moon_halo_full"] },
    });

    await userEvent.click(getByRole("button", { name: "Sun" }));
    await findByRole("dialog", { name: "Sun" });
    await userEvent.click(getByRole("checkbox", { name: "Lens flares" }));
    await userEvent.click(getByRole("checkbox", { name: "Sunshafts" }));

    expect(view.options.isLensFlared).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isLensFlared);
    expect(view.options.isSunShafted).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isSunShafted);

    await userEvent.click(getByRole("combobox", { name: "sun" }));
    await userEvent.click(await findByRole("option", { name: "moon_halo_full" }));

    expect(weather.manual?.sun).toBe("moon_halo_full");
  });

  it("sets the level's sun shafts' quality and the floor under their density", async () => {
    const { getByRole, findByRole, weather } = await renderAction(<LevelSunAction />);

    await userEvent.click(getByRole("button", { name: "Sun" }));
    await findByRole("dialog", { name: "Sun" });
    await userEvent.click(getByRole("button", { name: "Low" }));

    expect(weather.sunShafts).toEqual({ minimum: 0, quality: ERenderSunShaftsQuality.LOW });
    expect(getByRole("slider", { name: "Minimum" })).toHaveAttribute("aria-valuemax", "0.5");
  });

  // The froxel volume replaces the sun's shafts while enhanced, so its settings sit beside them.
  it("sets the view's volumetric lighting from the sun's settings", async () => {
    const { getByRole, findByRole, view } = await renderAction(<LevelSunAction />);

    await userEvent.click(getByRole("button", { name: "Sun" }));
    await findByRole("dialog", { name: "Sun" });
    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.volumetric).toEqual({ mode: ERenderVolumetricMode.ENHANCED });
  });

  it("draws the fog enhanced on asking, offering its strengths only then, and back to the settings", async () => {
    const { getByRole, findByRole, queryByRole, view } = await renderAction(<LevelFogAction />);

    await open(getByRole, "Fog");
    await findByRole("dialog", { name: "Fog" });

    expect(queryByRole("slider", { name: "Height" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.fog).toEqual({ mode: ERenderFogMode.ENHANCED });
    expect(getByRole("slider", { name: "Height" })).toHaveAttribute("aria-valuetext", "8.0 m");
    expect(getByRole("slider", { name: "Density" })).toHaveAttribute("aria-valuetext", "+40%");
    expect(getByRole("slider", { name: "Scattering" })).toHaveAttribute("aria-valuetext", "0%");

    await userEvent.click(getByRole("button", { name: "Back to the settings for the fog" }));

    expect(view.features.fog).toEqual({});
  });

  it("debands the sky on asking, offering its quality and radius only then", async () => {
    const { getByRole, findByRole, queryByRole, view } = await renderAction(<LevelSkyAction />);

    await userEvent.click(getByRole("button", { name: "Sky" }));
    await findByRole("dialog", { name: "Sky" });

    expect(queryByRole("slider", { name: "Radius" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Debanding" }));

    expect(view.features.debanding).toEqual({ mode: ERenderDebandingMode.ENHANCED });
    expect(getByRole("slider", { name: "Radius" })).toHaveAttribute("aria-valuetext", "48 px");

    await userEvent.click(getByRole("button", { name: "Ultra" }));

    expect(view.features.debanding).toEqual({
      mode: ERenderDebandingMode.ENHANCED,
      quality: ERenderDebandingQuality.ULTRA,
    });
  });
});
