import { useInjection } from "@wirestate/react";
import { formatPercent } from "@xrf/format";
import { ERenderWaterMode } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverNote } from "@xrf/ui/editor/popover-row";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { WaterIcon } from "@xrf/ui/icons/water";
import { ReactElement, useCallback } from "react";

import { LevelSettingsResetButton } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { describeLevelFeatureToggle, toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_WATER_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import {
  RENDER_ENHANCED_WATER_SETTINGS,
  RENDER_WATER_SETTINGS,
} from "@/core/render/lib/features/world/render-water-choices";
import {
  TRenderEnhancedWaterSettings,
  TRenderReflectionSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * Whether this view draws the water, how strongly the weather has it reflect, and how it moves, reflects and
 * distorts: the engine's own, or the enhanced water refracting what lies under it.
 */
export function LevelWaterAction({
  "data-testid": dataTestId = "level-water-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isWaterVisible");
  const { state, set, setNested, reset } = useLevelFeature("water");
  // The keyframe on screen, whose `water_intensity` scales what the water reflects.
  const manual: ILevelManualWeather = weatherService.shown;
  const water: TRenderWaterSettings = state.value;
  const isEnhanced: boolean = water.mode === ERenderWaterMode.ENHANCED;
  // The water's reflection is set in the Reflections group, beside the surfaces' and the ray trace both share.
  const reflections: TRenderReflectionSettings = toLevelModeFeature("reflections", settings, features);
  const isReflectionTraced: boolean = renderService.drawing.isTraceable && reflections.isTraced;

  const onSetEnhanced = useCallback(
    (part: Partial<TRenderEnhancedWaterSettings>): void => setNested("enhanced", part),
    [setNested]
  );

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Water"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        label: "Water",
        off: "Water off, what lies under it showing",
        on: isEnhanced
          ? `Enhanced water refracting what lies under it${water.isDistorted ? ", distorting" : ""}`
          : `Water reflecting the sky${water.isSoft ? ", soft" : ""}${water.isDistorted ? ", distorting" : ""}`,
      })}
      icon={<WaterIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Water"}
      onToggle={toggle}
    >
      <LevelManualWeatherSlider
        field={"waterIntensity"}
        manual={manual}
        format={formatPercent}
        onEdit={weatherService.editManual}
      />

      <LevelWeatherResetButton keys={LEVEL_MANUAL_WEATHER_WATER_KEYS} onReset={weatherService.resetManual} />

      <RenderSettingEntries
        settings={RENDER_WATER_SETTINGS}
        names={["mode"]}
        group={water}
        features={drawn}
        onSet={set}
      />

      <RenderSettingEntries
        settings={RENDER_ENHANCED_WATER_SETTINGS}
        names={["refraction", "turbidity", "softBorder", "reflectivity"]}
        group={water.enhanced}
        features={drawn}
        onSet={onSetEnhanced}
      />

      {isEnhanced ? (
        <EditorPopoverNote>
          {!reflections.isWaterReflected
            ? "It reflects the sky's cube alone: Reflections, Water reflects the screen."
            : isReflectionTraced
              ? "It reflects the screen, what that misses ray-traced: Reflections, Water and Ray tracing."
              : "It reflects the screen, the sky's cube where that misses: Reflections, Water and Ray tracing."}
        </EditorPopoverNote>
      ) : null}

      <RenderSettingEntries
        settings={RENDER_ENHANCED_WATER_SETTINGS}
        names={[
          "reflectionBlur",
          "blurNoise",
          "vibrance",
          "specular",
          "caustics",
          "parallaxHeight",
          "flow",
          "calmFlow",
          "variation",
          "ripples",
        ]}
        group={water.enhanced}
        features={drawn}
        onSet={onSetEnhanced}
      />

      <RenderSettingEntries
        settings={RENDER_WATER_SETTINGS}
        names={["isSoft", "isDistorted", "waveHeight", "waveSpeed", "ripple", "reflection", "distortion"]}
        group={water}
        features={drawn}
        onSet={set}
      />

      <LevelSettingsResetButton onReset={reset} />
    </EditorPopoverToggle>
  );
}
