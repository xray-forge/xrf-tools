import { default as LightIcon } from "@mui/icons-material/Light";
import { Button } from "@mui/material";
import { ERendererLightShadowFilter, IRendererLightsSettings } from "@xrf/renderer";
import { ReactElement } from "react";

import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RENDER_LIGHT_SHADOW_FILTER_OPTIONS } from "@/core/render/lib/features/render-feature-choices";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelLightsActionProps extends BaseComponentProps {
  isOn: boolean;
  /** The lights the view is drawn with: the settings', with the view's own values over them. */
  lights: IRendererLightsSettings;
  /** Whether the renderer's settings draw lights at all, which this view can only narrow. */
  isAvailable?: boolean;
  features: ILevelFeatureOptions;
  onToggle: () => void;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * Whether this view lights the level with its lamps, and with the level file's own lights too.
 */
export function LevelLightsAction({
  "data-testid": dataTestId = "level-lights-action",
  id,
  className,
  isOn,
  lights,
  isAvailable = true,
  features,
  onToggle,
  onChange,
}: ILevelLightsActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Lights"}
      description={
        !isAvailable
          ? "Lights are off in Settings, under Rendering"
          : isOn
            ? `The level's lamps${lights.isLevelLights ? " and the level file's own lights" : ""}, ${lights.isShadowed ? "shadowed" : "unshadowed"}`
            : "Lights off, only the sun and the baked light"
      }
      icon={<LightIcon />}
      isOn={isOn && isAvailable}
      isDisabled={!isAvailable}
      toggleLabel={"Light the lamps"}
      onToggle={onToggle}
    >
      <CheckboxFormRow
        label={"Shadows"}
        isChecked={lights.isShadowed}
        onChange={(isShadowed: boolean) => onChange({ ...features, lights: { ...features.lights, isShadowed } })}
      />

      <RenderValueChoice
        label={"Shadow filter"}
        options={RENDER_LIGHT_SHADOW_FILTER_OPTIONS}
        value={lights.shadowFilter}
        onChange={(shadowFilter: ERendererLightShadowFilter) =>
          onChange({ ...features, lights: { ...features.lights, shadowFilter } })
        }
      />

      <CheckboxFormRow
        label={"Level lights: r2_allow_r1_lights"}
        isChecked={lights.isLevelLights}
        onChange={(isLevelLights: boolean) => onChange({ ...features, lights: { ...features.lights, isLevelLights } })}
      />

      <Button size={"small"} onClick={() => onChange({ ...features, lights: {} })}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
