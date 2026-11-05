import { useInjection } from "@wirestate/react";
import { formatPercent } from "@xrf/format";
import { ERenderWaterMode } from "@xrf/ipc/types/xrf-renderer";
import { Optional } from "@xrf/types";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { WaterIcon } from "@xrf/ui/icons/water";
import { ReactElement, useCallback } from "react";

import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import {
  LEVEL_SETTINGS_RESET_LABEL,
  useLevelFeatureReset,
} from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import {
  LEVEL_WEATHER_RESET_LABEL,
  useLevelWeatherReset,
} from "@/core/level/components/weather/use-level-weather-reset";
import { describeLevelFeatureToggle, toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_WATER_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RenderSettingSection } from "@/core/render/components/settings/RenderSettingSection";
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
 * Whether this view draws the water, the engine's own or the enhanced water refracting what lies under it, and a
 * section each of its waves, of what shows under its surface, of its reflection and of its distortion, the enhanced
 * water's values disabled under the engine's; then how strongly the weather has it reflect.
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
  const { state, set, setNested } = useLevelFeature("water");
  const onReset: Optional<() => void> = useLevelFeatureReset("water");
  const onWeatherReset: Optional<() => void> = useLevelWeatherReset(LEVEL_MANUAL_WEATHER_WATER_KEYS);
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
      resetLabel={LEVEL_SETTINGS_RESET_LABEL}
      onToggle={toggle}
      onReset={onReset}
    >
      <RenderSettingEntries
        settings={RENDER_WATER_SETTINGS}
        names={["mode", "isSoft"]}
        group={water}
        features={drawn}
        onSet={set}
      />

      <EditorPopoverGroupSection label={"Waves"}>
        <RenderSettingEntries
          settings={RENDER_WATER_SETTINGS}
          names={["waveHeight", "waveSpeed", "ripple"]}
          group={water}
          features={drawn}
          onSet={set}
        />

        <RenderSettingEntries
          settings={RENDER_ENHANCED_WATER_SETTINGS}
          names={["parallaxHeight", "flow", "calmFlow", "variation", "ripples"]}
          group={water.enhanced}
          features={drawn}
          onSet={onSetEnhanced}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection label={"Under the surface"}>
        <RenderSettingEntries
          settings={RENDER_ENHANCED_WATER_SETTINGS}
          names={["refraction", "turbidity", "softBorder", "caustics"]}
          group={water.enhanced}
          features={drawn}
          onSet={onSetEnhanced}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Reflection"}
        note={
          isEnhanced
            ? !reflections.isWaterReflected
              ? "It reflects the sky's cube alone: Reflections, Water reflects the screen."
              : isReflectionTraced
                ? "It reflects the screen, what that misses ray-traced: Reflections, Water and Ray tracing."
                : "It reflects the screen, the sky's cube where that misses: Reflections, Water and Ray tracing."
            : undefined
        }
      >
        <RenderSettingEntries
          settings={RENDER_WATER_SETTINGS}
          names={["reflection"]}
          group={water}
          features={drawn}
          onSet={set}
        />

        <RenderSettingEntries
          settings={RENDER_ENHANCED_WATER_SETTINGS}
          names={["reflectivity", "reflectionBlur", "blurNoise", "specular", "vibrance"]}
          group={water.enhanced}
          features={drawn}
          onSet={onSetEnhanced}
        />
      </EditorPopoverGroupSection>

      <RenderSettingSection
        settings={RENDER_WATER_SETTINGS}
        name={"isDistorted"}
        names={["distortion"]}
        group={water}
        features={drawn}
        onSet={set}
      />

      <EditorPopoverGroupSection label={"Weather"} resetLabel={LEVEL_WEATHER_RESET_LABEL} onReset={onWeatherReset}>
        <LevelManualWeatherSlider
          field={"waterIntensity"}
          label={"Intensity"}
          manual={manual}
          format={formatPercent}
          onEdit={weatherService.editManual}
        />
      </EditorPopoverGroupSection>
    </EditorPopoverToggle>
  );
}
