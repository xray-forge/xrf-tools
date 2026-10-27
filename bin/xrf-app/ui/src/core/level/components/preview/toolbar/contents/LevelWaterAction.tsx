import { default as WaterIcon } from "@mui/icons-material/Water";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatPercent } from "@xrf/format";
import { ERenderWaterMode, RenderWaterMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement, useCallback } from "react";

import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { describeLevelFeatureToggle, toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_WATER_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelRenderService } from "@/core/level/services/level-render/level-render.service";
import { LevelWeatherService } from "@/core/level/services/level-weather/level-weather.service";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderWaterMode,
  formatWaterBorder,
  formatWaterDistortion,
  formatWaterMultiple,
  formatWaterStrength,
  formatWaveHeight,
  formatWaveSpeed,
  RENDER_ENHANCED_WATER_LIMITS,
  RENDER_WATER_LIMITS,
  RENDER_WATER_MODE_OPTIONS,
} from "@/core/render/lib/features/render-water-choices";
import {
  TRenderEnhancedWaterSettings,
  TRenderReflectionSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
  const renderFeatures = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isWaterVisible");
  const { state, set, setNested, reset } = useLevelFeature("water");
  // The keyframe on screen, whose `water_intensity` scales what the water reflects.
  const manual: ILevelManualWeather = weatherService.shown;
  const water: TRenderWaterSettings = state.value;
  const enhanced: TRenderEnhancedWaterSettings = water.enhanced;
  const isEnhanced: boolean = water.mode === ERenderWaterMode.ENHANCED;
  // The water's reflection is set in the Reflections group, beside the surfaces' and the ray trace both share.
  const reflections: TRenderReflectionSettings = toLevelModeFeature(
    "reflections",
    renderFeatures.settings,
    renderFeatures.features
  );
  const isReflectionTraced: boolean = renderService.isTraceable && reflections.isTraced;

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

      <RenderValueChoice
        label={"Mode"}
        options={RENDER_WATER_MODE_OPTIONS}
        value={water.mode}
        onChange={(mode: RenderWaterMode) => set({ mode })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderWaterMode(water.mode)}</p>

      <RenderValueSlider
        label={"Refraction"}
        value={enhanced.refraction}
        {...RENDER_ENHANCED_WATER_LIMITS.refraction}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(refraction: number) => onSetEnhanced({ refraction })}
      />

      <RenderValueSlider
        label={"Turbidity"}
        value={enhanced.turbidity}
        {...RENDER_ENHANCED_WATER_LIMITS.turbidity}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(turbidity: number) => onSetEnhanced({ turbidity })}
      />

      <RenderValueSlider
        label={"Soft border"}
        value={enhanced.softBorder}
        {...RENDER_ENHANCED_WATER_LIMITS.softBorder}
        isDisabled={!isEnhanced}
        format={formatWaterBorder}
        onChange={(softBorder: number) => onSetEnhanced({ softBorder })}
      />

      <RenderValueSlider
        label={"Reflectivity"}
        value={enhanced.reflectivity}
        {...RENDER_ENHANCED_WATER_LIMITS.reflectivity}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(reflectivity: number) => onSetEnhanced({ reflectivity })}
      />

      {isEnhanced ? (
        <p className={"text-xs text-text-secondary"}>
          {!reflections.isWaterReflected
            ? "It reflects the sky's cube alone: Reflections, Water reflects the screen."
            : isReflectionTraced
              ? "It reflects the screen, what that misses ray-traced: Reflections, Water and Ray tracing."
              : "It reflects the screen, the sky's cube where that misses: Reflections, Water and Ray tracing."}
        </p>
      ) : null}

      <RenderValueSlider
        label={"Reflection blur"}
        value={enhanced.reflectionBlur}
        {...RENDER_ENHANCED_WATER_LIMITS.reflectionBlur}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(reflectionBlur: number) => onSetEnhanced({ reflectionBlur })}
      />

      <RenderValueSlider
        label={"Blur noise"}
        value={enhanced.blurNoise}
        {...RENDER_ENHANCED_WATER_LIMITS.blurNoise}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(blurNoise: number) => onSetEnhanced({ blurNoise })}
      />

      <RenderValueSlider
        label={"Vibrance"}
        value={enhanced.vibrance}
        {...RENDER_ENHANCED_WATER_LIMITS.vibrance}
        isDisabled={!isEnhanced}
        format={formatPercent}
        onChange={(vibrance: number) => onSetEnhanced({ vibrance })}
      />

      <RenderValueSlider
        label={"Specular"}
        value={enhanced.specular}
        {...RENDER_ENHANCED_WATER_LIMITS.specular}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(specular: number) => onSetEnhanced({ specular })}
      />

      <RenderValueSlider
        label={"Caustics"}
        value={enhanced.caustics}
        {...RENDER_ENHANCED_WATER_LIMITS.caustics}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(caustics: number) => onSetEnhanced({ caustics })}
      />

      <RenderValueSlider
        label={"Parallax height"}
        value={enhanced.parallaxHeight}
        {...RENDER_ENHANCED_WATER_LIMITS.parallaxHeight}
        isDisabled={!isEnhanced}
        format={formatWaveHeight}
        onChange={(parallaxHeight: number) => onSetEnhanced({ parallaxHeight })}
      />

      <RenderValueSlider
        label={"Flow"}
        value={enhanced.flow}
        {...RENDER_ENHANCED_WATER_LIMITS.flow}
        isDisabled={!isEnhanced}
        format={formatWaterMultiple}
        onChange={(flow: number) => onSetEnhanced({ flow })}
      />

      <RenderValueSlider
        label={"Calm flow"}
        value={enhanced.calmFlow}
        {...RENDER_ENHANCED_WATER_LIMITS.calmFlow}
        isDisabled={!isEnhanced}
        format={formatPercent}
        onChange={(calmFlow: number) => onSetEnhanced({ calmFlow })}
      />

      <RenderValueSlider
        label={"Variation"}
        value={enhanced.variation}
        {...RENDER_ENHANCED_WATER_LIMITS.variation}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(variation: number) => onSetEnhanced({ variation })}
      />

      <RenderValueSlider
        label={"Rain ripples"}
        value={enhanced.ripples}
        {...RENDER_ENHANCED_WATER_LIMITS.ripples}
        isDisabled={!isEnhanced}
        format={formatWaterStrength}
        onChange={(ripples: number) => onSetEnhanced({ ripples })}
      />

      <CheckboxFormRow
        label={"Soft water"}
        description={"Fades the water by its depth and lays foam in the shallows. Game setting: r2_soft_water."}
        isChecked={water.isSoft}
        onChange={(isSoft: boolean) => set({ isSoft })}
      />

      <CheckboxFormRow
        label={"Distortion"}
        description={"Moves what is seen through it."}
        isChecked={water.isDistorted}
        onChange={(isDistorted: boolean) => set({ isDistorted })}
      />

      <RenderValueSlider
        label={"Wave height"}
        value={water.waveHeight}
        {...RENDER_WATER_LIMITS.waveHeight}
        format={formatWaveHeight}
        onChange={(waveHeight: number) => set({ waveHeight })}
      />

      <RenderValueSlider
        label={"Wave speed"}
        value={water.waveSpeed}
        {...RENDER_WATER_LIMITS.waveSpeed}
        format={formatWaveSpeed}
        onChange={(waveSpeed: number) => set({ waveSpeed })}
      />

      <RenderValueSlider
        label={"Ripple"}
        value={water.ripple}
        {...RENDER_WATER_LIMITS.ripple}
        format={formatWaterMultiple}
        onChange={(ripple: number) => set({ ripple })}
      />

      <RenderValueSlider
        label={"Reflection"}
        value={water.reflection}
        {...RENDER_WATER_LIMITS.reflection}
        format={formatWaterMultiple}
        onChange={(reflection: number) => set({ reflection })}
      />

      <RenderValueSlider
        label={"Distortion strength"}
        value={water.distortion}
        {...RENDER_WATER_LIMITS.distortion}
        format={formatWaterDistortion}
        onChange={(distortion: number) => set({ distortion })}
      />

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </EditorPopoverToggle>
  );
}
