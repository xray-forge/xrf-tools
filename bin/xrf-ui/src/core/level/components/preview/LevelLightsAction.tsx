import { default as LightIcon } from "@mui/icons-material/Light";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { RenderLightShadowFilter } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RENDER_LIGHT_SHADOW_FILTER_OPTIONS } from "@/core/render/lib/features";
import { TRenderLightsSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";

/**
 * Whether this view lights the level with its lamps, and with the level file's own lights too.
 */
export function LevelLightsAction({
  "data-testid": dataTestId = "level-lights-action",
  id,
  className,
  isOn,
  state,
  features,
  onToggle,
  onChange,
}: ILevelFeatureActionProps<"lights">): ReactElement {
  const { set, reset } = useLevelFeatureOverride("lights", features, onChange);
  const lights: TRenderLightsSettings = state.value;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Lights"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        isPlural: true,
        label: "Lights",
        off: "Lights off, only the sun and the baked light",
        on:
          `The level's lamps${lights.isLevelLights ? " and the level file's own lights" : ""}, ` +
          (lights.isShadowed ? "shadowed" : "unshadowed"),
      })}
      icon={<LightIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Light the lamps"}
      onToggle={onToggle}
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
        label={"Level lights"}
        description={"The level file's own, which the game draws only with r2_allow_r1_lights."}
        isChecked={lights.isLevelLights}
        onChange={(isLevelLights: boolean) => set({ isLevelLights })}
      />

      <Button size={"small"} onClick={reset}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
