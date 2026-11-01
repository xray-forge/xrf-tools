import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { LightIcon } from "@xrf/ui/icons/light";
import { ReactElement } from "react";

import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import { LevelSettingsResetButton } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle } from "@/core/level/lib/features/level-feature-options";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RENDER_LIGHTS_SETTINGS } from "@/core/render/lib/features/lighting/render-lights-choices";
import { TRenderLightsSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

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
  const { drawn } = useLevelRenderFeatures();

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
      <RenderSettingEntries
        settings={RENDER_LIGHTS_SETTINGS}
        names={["isShadowed", "shadowFilter", "falloff", "isVolumetric", "shafts", "shaftStrength", "isLevelLights"]}
        group={lights}
        features={drawn}
        onSet={set}
      />

      <LevelSettingsResetButton onReset={reset} />

      <LevelEnhancedFeatureSection feature={"volumetric"} />
    </EditorPopoverToggle>
  );
}
