import { default as WaterIcon } from "@mui/icons-material/Water";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderWaterMode, RenderWaterMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatWaterBorder,
  formatWaterDistortion,
  formatWaterMultiple,
  formatWaterStrength,
  formatWaveHeight,
  formatWaveSpeed,
  IRenderChoiceOption,
  RENDER_ENHANCED_WATER_LIMITS,
  RENDER_WATER_LIMITS,
} from "@/core/render/lib/features";
import { TRenderEnhancedWaterSettings, TRenderWaterSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { formatPercent } from "@/lib/format/number";

const MODE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderWaterMode>> = [
  { label: "Engine", value: ERenderWaterMode.ENGINE },
  { label: "Enhanced", value: ERenderWaterMode.ENHANCED },
];

interface ILevelWaterActionProps extends ILevelFeatureActionProps<"water"> {
  /** The keyframe on screen, whose `water_intensity` scales what the water reflects. */
  manual: ILevelManualWeather;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}

/**
 * Whether this view draws the water, how strongly the weather has it reflect, and how it moves, reflects and
 * distorts: the engine's own, or the enhanced water refracting what lies under it.
 */
export function LevelWaterAction({
  "data-testid": dataTestId = "level-water-action",
  id,
  className,
  isOn,
  state,
  features,
  manual,
  onToggle,
  onChange,
  onEdit,
}: ILevelWaterActionProps): ReactElement {
  const { set, reset } = useLevelFeatureOverride("water", features, onChange);
  const water: TRenderWaterSettings = state.value;
  const enhanced: TRenderEnhancedWaterSettings = water.enhanced;

  const setEnhanced = useCallback(
    (part: Partial<TRenderEnhancedWaterSettings>): void => {
      set({ enhanced: { ...enhanced, ...part } });
    },
    [enhanced, set]
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
        on:
          water.mode === ERenderWaterMode.ENHANCED
            ? `Enhanced water refracting what lies under it${water.isDistorted ? ", distorting" : ""}`
            : `Water reflecting the sky${water.isSoft ? ", soft" : ""}${water.isDistorted ? ", distorting" : ""}`,
      })}
      icon={<WaterIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Draw the water"}
      onToggle={onToggle}
    >
      <LevelManualWeatherSlider field={"waterIntensity"} manual={manual} format={formatPercent} onEdit={onEdit} />

      <RenderValueChoice
        label={"Mode"}
        options={MODE_OPTIONS}
        value={water.mode}
        onChange={(mode: RenderWaterMode) => set({ mode })}
      />

      {water.mode === ERenderWaterMode.ENHANCED ? (
        <>
          <RenderValueSlider
            label={"Refraction"}
            value={enhanced.refraction}
            {...RENDER_ENHANCED_WATER_LIMITS.refraction}
            format={formatWaterStrength}
            onChange={(refraction: number) => setEnhanced({ refraction })}
          />

          <RenderValueSlider
            label={"Turbidity"}
            value={enhanced.turbidity}
            {...RENDER_ENHANCED_WATER_LIMITS.turbidity}
            format={formatWaterStrength}
            onChange={(turbidity: number) => setEnhanced({ turbidity })}
          />

          <RenderValueSlider
            label={"Soft border"}
            value={enhanced.softBorder}
            {...RENDER_ENHANCED_WATER_LIMITS.softBorder}
            format={formatWaterBorder}
            onChange={(softBorder: number) => setEnhanced({ softBorder })}
          />

          <RenderValueSlider
            label={"Reflectivity"}
            value={enhanced.reflectivity}
            {...RENDER_ENHANCED_WATER_LIMITS.reflectivity}
            format={formatWaterStrength}
            onChange={(reflectivity: number) => setEnhanced({ reflectivity })}
          />

          <RenderValueSlider
            label={"Reflection blur"}
            value={enhanced.reflectionBlur}
            {...RENDER_ENHANCED_WATER_LIMITS.reflectionBlur}
            format={formatWaterStrength}
            onChange={(reflectionBlur: number) => setEnhanced({ reflectionBlur })}
          />

          <RenderValueSlider
            label={"Blur noise"}
            value={enhanced.blurNoise}
            {...RENDER_ENHANCED_WATER_LIMITS.blurNoise}
            format={formatWaterStrength}
            onChange={(blurNoise: number) => setEnhanced({ blurNoise })}
          />

          <RenderValueSlider
            label={"Specular"}
            value={enhanced.specular}
            {...RENDER_ENHANCED_WATER_LIMITS.specular}
            format={formatWaterStrength}
            onChange={(specular: number) => setEnhanced({ specular })}
          />

          <RenderValueSlider
            label={"Caustics"}
            value={enhanced.caustics}
            {...RENDER_ENHANCED_WATER_LIMITS.caustics}
            format={formatWaterStrength}
            onChange={(caustics: number) => setEnhanced({ caustics })}
          />

          <RenderValueSlider
            label={"Parallax height"}
            value={enhanced.parallaxHeight}
            {...RENDER_ENHANCED_WATER_LIMITS.parallaxHeight}
            format={formatWaveHeight}
            onChange={(parallaxHeight: number) => setEnhanced({ parallaxHeight })}
          />

          <RenderValueSlider
            label={"Flow"}
            value={enhanced.flow}
            {...RENDER_ENHANCED_WATER_LIMITS.flow}
            format={formatWaterMultiple}
            onChange={(flow: number) => setEnhanced({ flow })}
          />

          <RenderValueSlider
            label={"Calm flow"}
            value={enhanced.calmFlow}
            {...RENDER_ENHANCED_WATER_LIMITS.calmFlow}
            format={formatPercent}
            onChange={(calmFlow: number) => setEnhanced({ calmFlow })}
          />

          <RenderValueSlider
            label={"Variation"}
            value={enhanced.variation}
            {...RENDER_ENHANCED_WATER_LIMITS.variation}
            format={formatWaterStrength}
            onChange={(variation: number) => setEnhanced({ variation })}
          />

          <RenderValueSlider
            label={"Rain ripples"}
            value={enhanced.ripples}
            {...RENDER_ENHANCED_WATER_LIMITS.ripples}
            format={formatWaterStrength}
            onChange={(ripples: number) => setEnhanced({ ripples })}
          />
        </>
      ) : null}

      <CheckboxFormRow
        label={"Soft"}
        description={"Fades by how deep it is, and lays foam in the shallows: r2_soft_water."}
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
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
