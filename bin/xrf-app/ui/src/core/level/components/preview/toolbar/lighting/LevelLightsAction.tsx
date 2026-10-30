import { RenderLightFalloff, RenderLightShadowFilter, RenderLightShafts } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { LightIcon } from "@xrf/ui/icons/light";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderLightFalloff,
  RENDER_LIGHT_FALLOFF_OPTIONS,
} from "@/core/render/lib/features/render-light-falloff-choices";
import {
  explainRenderLightShafts,
  formatShaftStrength,
  RENDER_LIGHT_SHAFTS_OPTIONS,
  RENDER_SHAFT_STRENGTH_LIMITS,
} from "@/core/render/lib/features/render-light-shafts-choices";
import { RENDER_LIGHT_SHADOW_FILTER_OPTIONS } from "@/core/render/lib/features/render-shadow-choices";
import { TRenderLightsSettings } from "@/core/render/lib/settings/render-feature-settings";

/**
 * Whether this view lights the level with its lamps, and with the level file's own lights too; which lamps draw shafts
 * how strongly; and how the light the air scatters is drawn.
 */
export function LevelLightsAction({
  "data-testid": dataTestId = "level-lights-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { isOn, toggle } = useLevelViewToggle("isLamplit");
  const { state, set, reset } = useLevelFeature("lights");

  const lights: TRenderLightsSettings = state.value;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Lamps"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        isPlural: true,
        label: "Lamps",
        off: "Lamps off: only the sun and the baked light",
        on:
          `The level's lamps${lights.isLevelLights ? " and the level file's own lights" : ""}, ` +
          (lights.isShadowed ? "shadowed" : "unshadowed"),
      })}
      icon={<LightIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Lamps"}
      onToggle={toggle}
    >
      <CheckboxFormRow
        label={"Shadows"}
        isChecked={lights.isShadowed}
        onChange={(isShadowed: boolean) => set({ isShadowed })}
      />

      <RenderValueChoice
        label={"Shadow filter"}
        options={RENDER_LIGHT_SHADOW_FILTER_OPTIONS}
        value={lights.shadowFilter}
        onChange={(shadowFilter: RenderLightShadowFilter) => set({ shadowFilter })}
      />

      <RenderValueChoice
        label={"Falloff"}
        options={RENDER_LIGHT_FALLOFF_OPTIONS}
        value={lights.falloff}
        onChange={(falloff: RenderLightFalloff) => set({ falloff })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderLightFalloff(lights.falloff)}</p>

      <CheckboxFormRow
        label={"Lamp shafts"}
        description={"Shafts of light from the shadowed lamps. Game setting: r2_volumetric_lights."}
        isChecked={lights.isVolumetric}
        onChange={(isVolumetric: boolean) => set({ isVolumetric })}
      />

      <RenderValueChoice
        label={"Shafts from"}
        options={RENDER_LIGHT_SHAFTS_OPTIONS}
        value={lights.shafts}
        isDisabled={!lights.isVolumetric}
        onChange={(shafts: RenderLightShafts) => set({ shafts })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderLightShafts(lights.shafts)}</p>

      <RenderValueSlider
        label={"Shaft strength"}
        value={lights.shaftStrength}
        {...RENDER_SHAFT_STRENGTH_LIMITS}
        isDisabled={!lights.isVolumetric}
        format={formatShaftStrength}
        onChange={(shaftStrength: number) => set({ shaftStrength })}
      />

      <CheckboxFormRow
        label={"Level lights"}
        description={
          "The level file's own lights, which the game draws only with this on. Game setting: r2_allow_r1_lights."
        }
        isChecked={lights.isLevelLights}
        onChange={(isLevelLights: boolean) => set({ isLevelLights })}
      />

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>

      <LevelEnhancedFeatureSection feature={"volumetric"} />
    </EditorPopoverToggle>
  );
}
