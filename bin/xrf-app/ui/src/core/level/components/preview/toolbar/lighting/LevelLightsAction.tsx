import { RenderLightShadowFilter } from "@xrf/ipc/types/xrf-renderer";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { LightIcon } from "@xrf/ui/icons";
import { Button } from "@xrf/ui/material";
import { ReactElement } from "react";

import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RENDER_LIGHT_SHADOW_FILTER_OPTIONS } from "@/core/render/lib/features/render-shadow-choices";
import { TRenderLightsSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";

/**
 * Whether this view lights the level with its lamps, and with the level file's own lights too; and how the light the
 * air scatters is drawn.
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

      <CheckboxFormRow
        label={"Lamp shafts"}
        description={"Shafts of light from the shadowed lamps. Game setting: r2_volumetric_lights."}
        isChecked={lights.isVolumetric}
        onChange={(isVolumetric: boolean) => set({ isVolumetric })}
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
