import { useInjection } from "@wirestate/react";
import { formatPercent, formatSeconds } from "@xrf/format";
import { ThunderboltCollection } from "@xrf/ipc/types/xrf-environment";
import { ERenderRainDropsMode, ERenderRainMode } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { WaterDropIcon } from "@xrf/ui/icons/water-drop";
import { ReactElement } from "react";

import { LevelSettingsResetButton } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_RAIN_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
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

/** What the thunder's list offers before the level's weather is read. */
const NO_COLLECTIONS: ReadonlyArray<ThunderboltCollection> = [];

/**
 * How hard it rains, the streaks' colour, whether the drops bend the scene behind them, how far about the camera
 * they fall and how far, how strongly and how wide their rings ripple, the bolts that strike with it, and whether puddles gather on flat ground over the engine's own wetting: how many of the sites hold one, how much
 * they reflect, how far they are drawn and how strongly the rain ripples them; whether any of it happens at all.
 */
export function LevelRainAction({
  "data-testid": dataTestId = "level-rain-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set: setPuddles, reset: resetPuddles } = useLevelFeatureOverride("rain");

  const { isRainy, isThundering } = viewService.options;

  // The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from.
  const manual: ILevelManualWeather = weatherService.shown;
  const collections: ReadonlyArray<ThunderboltCollection> =
    weatherService.description.value?.thunderboltCollections ?? NO_COLLECTIONS;
  // The rain's wetting as the view draws it: whether puddles gather, and their strengths.
  const puddles: TRenderRainSettings = toLevelModeFeature("rain", settings, features);
  const isRaining: boolean = isRainy && manual.rainDensity > 0;
  const isStriking: boolean = isThundering && manual.thunderboltCollection !== "";
  const isPuddled: boolean = puddles.mode === ERenderRainMode.ENHANCED;
  const isRefracted: boolean = puddles.drops === ERenderRainDropsMode.ENHANCED;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Rain"}
      description={[
        isRaining ? `Rain at ${formatPercent(manual.rainDensity)}` : isRainy ? "No rain" : "Rain off",
        isStriking ? `bolts of ${manual.thunderboltCollection}` : isThundering ? "no thunder" : "thunder off",
        isPuddled ? `puddles at ${formatRainShare(puddles.puddles)} of the sites` : "no puddles",
      ].join(", ")}
      icon={<WaterDropIcon />}
      isActive={isRaining || isStriking || isPuddled}
    >
      <EditorPopoverGroupSection label={"Rain"} isOn={isRainy} onToggle={() => viewService.toggleOption("isRainy")}>
        <LevelManualWeatherSlider
          field={"rainDensity"}
          manual={manual}
          format={formatPercent}
          onEdit={weatherService.editManual}
        />

        <LevelManualWeatherVectorField field={"rainColor"} isColor manual={manual} onEdit={weatherService.editManual} />

        <CheckboxFormRow
          label={RENDER_RAIN_SETTINGS.drops.label}
          description={explainRenderRainDropsMode(puddles.drops)}
          isChecked={isRefracted}
          onChange={(isOn: boolean) =>
            setPuddles({ drops: isOn ? ERenderRainDropsMode.ENHANCED : ERenderRainDropsMode.ENGINE })
          }
        />

        <RenderSettingEntries
          settings={RENDER_RAIN_SETTINGS}
          names={["dropSquare", "ringDistance", "ringStrength", "ringSize", "pace"]}
          group={puddles}
          features={drawn}
          onSet={setPuddles}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Thunder"}
        isOn={isThundering}
        onToggle={() => viewService.toggleOption("isThundering")}
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

      <LevelWeatherResetButton keys={LEVEL_MANUAL_WEATHER_RAIN_KEYS} onReset={weatherService.resetManual} />

      <EditorPopoverGroupSection
        label={"Puddles"}
        note={explainRenderRainMode(puddles.mode)}
        isOn={isPuddled}
        onToggle={() => setPuddles({ mode: isPuddled ? ERenderRainMode.ENGINE : ERenderRainMode.ENHANCED })}
      >
        <RenderSettingEntries
          settings={RENDER_RAIN_SETTINGS}
          names={["puddles", "reflectivity", "distance", "ripples", "isReliefFlooded"]}
          group={puddles}
          features={drawn}
          onSet={setPuddles}
        />

        <LevelSettingsResetButton onReset={resetPuddles} />
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
