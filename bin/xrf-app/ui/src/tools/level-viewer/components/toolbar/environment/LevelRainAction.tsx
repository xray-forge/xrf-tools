import { useInjection } from "@wirestate/react";
import { EMPTY_ARRAY } from "@xrf/collections";
import { formatPercent, formatSeconds } from "@xrf/format";
import { ThunderboltCollection } from "@xrf/ipc/types/xrf-environment";
import { ERenderRainDropsMode, ERenderRainMode } from "@xrf/ipc/types/xrf-renderer";
import { Optional } from "@xrf/types";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { WaterDropIcon } from "@xrf/ui/icons/water-drop";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import {
  LEVEL_SETTINGS_RESET_LABEL,
  useLevelFeatureReset,
} from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import {
  LEVEL_WEATHER_RESET_LABEL,
  useLevelWeatherReset,
} from "@/core/level/components/weather/use-level-weather-reset";
import { toLevelModeFeature, withoutLevelFeatureKeys } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  LEVEL_MANUAL_WEATHER_RAIN_KEYS,
  LEVEL_MANUAL_WEATHER_THUNDER_KEYS,
} from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import {
  explainRenderRainDropsMode,
  explainRenderRainMode,
  formatRainShare,
  RENDER_RAIN_SETTINGS,
} from "@/core/render/lib/features/world/render-rain-choices";
import { TRenderRainSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { LevelWeatherCollectionSelect } from "@/tools/level-viewer/components/weather/LevelWeatherCollectionSelect";

/** The rain settings the falling drops' section sets, and gives back to the settings. */
const RAIN_DROPS_KEYS: ReadonlyArray<keyof TRenderRainSettings> = [
  "drops",
  "dropSquare",
  "ringDistance",
  "ringStrength",
  "ringSize",
];

/**
 * The rain settings the puddles' section sets, and gives back to the settings: every other one, the wetting pace among
 * them, so each rain setting is given back by exactly one of the two sections.
 */
const RAIN_PUDDLES_KEYS: ReadonlyArray<keyof TRenderRainSettings> = (
  Object.keys(RENDER_RAIN_SETTINGS) as Array<keyof TRenderRainSettings>
).filter((key: keyof TRenderRainSettings) => !RAIN_DROPS_KEYS.includes(key));

/**
 * How hard it rains and the streaks' colour; whether the drops bend the scene behind them, how far about the camera
 * they fall, and how far, how strongly and how wide their rings ripple; the bolts that strike with it; whether puddles
 * gather on flat ground over the engine's own wetting, how many of the sites hold one, how much they reflect, how far
 * they are drawn and how strongly the rain ripples them. Each section resets its own part.
 */
export function LevelRainAction({
  "data-testid": dataTestId = "level-rain-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set } = useLevelFeatureOverride("rain");
  const onRainReset: Optional<() => void> = useLevelWeatherReset(LEVEL_MANUAL_WEATHER_RAIN_KEYS);
  const onThunderReset: Optional<() => void> = useLevelWeatherReset(LEVEL_MANUAL_WEATHER_THUNDER_KEYS);
  const onDropsReset: Optional<() => void> = useLevelFeatureReset(
    "rain",
    withoutLevelFeatureKeys(features.rain, RAIN_DROPS_KEYS)
  );
  const onPuddlesReset: Optional<() => void> = useLevelFeatureReset(
    "rain",
    withoutLevelFeatureKeys(features.rain, RAIN_PUDDLES_KEYS)
  );

  const { isRainy, isThundering } = viewService.options;

  // The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from.
  const manual: ILevelManualWeather = weatherService.shown;
  const collections: ReadonlyArray<ThunderboltCollection> =
    weatherService.description.value?.thunderboltCollections ?? EMPTY_ARRAY;
  // The rain as the view draws it: its drops, whether puddles gather, and their strengths.
  const rain: TRenderRainSettings = toLevelModeFeature("rain", settings, features);
  const isRaining: boolean = isRainy && manual.rainDensity > 0;
  const isStriking: boolean = isThundering && manual.thunderboltCollection !== "";
  const isPuddled: boolean = rain.mode === ERenderRainMode.ENHANCED;
  const isRefracted: boolean = rain.drops === ERenderRainDropsMode.ENHANCED;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Rain"}
      description={[
        isRaining ? `Rain at ${formatPercent(manual.rainDensity)}` : isRainy ? "No rain" : "Rain off",
        isStriking ? `bolts of ${manual.thunderboltCollection}` : isThundering ? "no thunder" : "thunder off",
        isPuddled ? `puddles at ${formatRainShare(rain.puddles)} of the sites` : "no puddles",
      ].join(", ")}
      icon={<WaterDropIcon />}
      isActive={isRaining || isStriking || isPuddled}
    >
      <EditorPopoverGroupSection
        label={"Rain"}
        isOn={isRainy}
        resetLabel={LEVEL_WEATHER_RESET_LABEL}
        onToggle={() => viewService.toggleOption("isRainy")}
        onReset={onRainReset}
      >
        <LevelManualWeatherSlider
          field={"rainDensity"}
          manual={manual}
          format={formatPercent}
          onEdit={weatherService.editManual}
        />

        <LevelManualWeatherVectorField field={"rainColor"} isColor manual={manual} onEdit={weatherService.editManual} />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={RENDER_RAIN_SETTINGS.drops.label}
        note={explainRenderRainDropsMode(rain.drops)}
        isOn={isRefracted}
        resetLabel={LEVEL_SETTINGS_RESET_LABEL}
        onToggle={() => set({ drops: isRefracted ? ERenderRainDropsMode.ENGINE : ERenderRainDropsMode.ENHANCED })}
        onReset={onDropsReset}
      >
        <RenderSettingEntries
          settings={RENDER_RAIN_SETTINGS}
          names={["dropSquare", "ringDistance", "ringStrength", "ringSize"]}
          group={rain}
          features={drawn}
          onSet={set}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Thunder"}
        isOn={isThundering}
        resetLabel={LEVEL_WEATHER_RESET_LABEL}
        onToggle={() => viewService.toggleOption("isThundering")}
        onReset={onThunderReset}
      >
        <LevelWeatherCollectionSelect
          value={manual.thunderboltCollection}
          collections={collections}
          onChange={(thunderboltCollection: string) => weatherService.editManual({ thunderboltCollection })}
        />

        <LevelManualWeatherSlider
          field={"thunderboltPeriod"}
          manual={manual}
          format={formatSeconds}
          onEdit={weatherService.editManual}
        />

        <LevelManualWeatherSlider
          field={"thunderboltDuration"}
          manual={manual}
          format={formatSeconds}
          onEdit={weatherService.editManual}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={RENDER_RAIN_SETTINGS.mode.label}
        note={explainRenderRainMode(rain.mode)}
        isOn={isPuddled}
        resetLabel={LEVEL_SETTINGS_RESET_LABEL}
        onToggle={() => set({ mode: isPuddled ? ERenderRainMode.ENGINE : ERenderRainMode.ENHANCED })}
        onReset={onPuddlesReset}
      >
        <RenderSettingEntries
          settings={RENDER_RAIN_SETTINGS}
          names={["puddles", "reflectivity", "distance", "ripples", "isReliefFlooded", "pace"]}
          group={rain}
          features={drawn}
          onSet={set}
        />
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
