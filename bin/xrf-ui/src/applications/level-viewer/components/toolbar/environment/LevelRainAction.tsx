import { default as WaterDropIcon } from "@mui/icons-material/WaterDrop";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatPercent, formatSeconds } from "@xrf/format";
import { ReactElement } from "react";

import { LevelWeatherCollectionSelect } from "@/applications/level-viewer/components/weather/LevelWeatherCollectionSelect";
import { ThunderboltCollection } from "@/core/ipc/types/xrf-environment";
import { ERenderRainDropsMode, ERenderRainMode } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_RAIN_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { formatGrassRadius } from "@/core/render/lib/features/render-grass-choices";
import {
  explainRenderRainDropsMode,
  explainRenderRainMode,
  formatRainPace,
  formatRainShare,
  RENDER_RAIN_LIMITS,
} from "@/core/render/lib/features/render-rain-choices";
import { TRenderRainSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
  const { settings, features } = useLevelRenderFeatures();
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
          label={"Refracting drops"}
          description={explainRenderRainDropsMode(puddles.drops)}
          isChecked={isRefracted}
          onChange={(isOn: boolean) =>
            setPuddles({ drops: isOn ? ERenderRainDropsMode.ENHANCED : ERenderRainDropsMode.ENGINE })
          }
        />

        {isRefracted ? (
          <>
            <RenderValueSlider
              label={"Drop area"}
              value={puddles.dropSquare}
              {...RENDER_RAIN_LIMITS.dropSquare}
              format={formatGrassRadius}
              onChange={(dropSquare: number) => setPuddles({ dropSquare })}
            />

            <RenderValueSlider
              label={"Ring distance"}
              value={puddles.ringDistance}
              {...RENDER_RAIN_LIMITS.ringDistance}
              format={formatGrassRadius}
              onChange={(ringDistance: number) => setPuddles({ ringDistance })}
            />

            <RenderValueSlider
              label={"Ring strength"}
              value={puddles.ringStrength}
              {...RENDER_RAIN_LIMITS.ringStrength}
              format={formatRainShare}
              onChange={(ringStrength: number) => setPuddles({ ringStrength })}
            />

            <RenderValueSlider
              label={"Ring size"}
              value={puddles.ringSize}
              {...RENDER_RAIN_LIMITS.ringSize}
              format={formatRainShare}
              onChange={(ringSize: number) => setPuddles({ ringSize })}
            />
          </>
        ) : null}

        <RenderValueSlider
          label={"Wetting pace"}
          value={puddles.pace}
          {...RENDER_RAIN_LIMITS.pace}
          format={formatRainPace}
          onChange={(pace: number) => setPuddles({ pace })}
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
        description={explainRenderRainMode(puddles.mode)}
        isOn={isPuddled}
        onToggle={() => setPuddles({ mode: isPuddled ? ERenderRainMode.ENGINE : ERenderRainMode.ENHANCED })}
      >
        {isPuddled ? (
          <>
            <RenderValueSlider
              label={"Puddles"}
              value={puddles.puddles}
              {...RENDER_RAIN_LIMITS.puddles}
              format={formatRainShare}
              onChange={(share: number) => setPuddles({ puddles: share })}
            />

            <RenderValueSlider
              label={"Reflectivity"}
              value={puddles.reflectivity}
              {...RENDER_RAIN_LIMITS.reflectivity}
              format={formatRainShare}
              onChange={(reflectivity: number) => setPuddles({ reflectivity })}
            />

            <RenderValueSlider
              label={"Distance"}
              value={puddles.distance}
              {...RENDER_RAIN_LIMITS.distance}
              format={formatGrassRadius}
              onChange={(distance: number) => setPuddles({ distance })}
            />

            <RenderValueSlider
              label={"Ripples"}
              value={puddles.ripples}
              {...RENDER_RAIN_LIMITS.ripples}
              format={formatRainShare}
              onChange={(ripples: number) => setPuddles({ ripples })}
            />

            <CheckboxFormRow
              label={"Fill the relief"}
              description={"Water fills the ground's hollows first where the enhanced parallax marches it."}
              isChecked={puddles.isReliefFlooded}
              onChange={(isReliefFlooded: boolean) => setPuddles({ isReliefFlooded })}
            />
          </>
        ) : null}

        <Button size={"small"} onClick={resetPuddles}>
          Reset to the preset
        </Button>
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
