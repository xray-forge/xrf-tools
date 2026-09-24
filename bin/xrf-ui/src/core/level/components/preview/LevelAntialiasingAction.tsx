import { default as DeblurIcon } from "@mui/icons-material/Deblur";
import { Button } from "@mui/material";
import { ERendererAntialiasing } from "@xrf/renderer";
import { ReactElement } from "react";

import { ILevelFeatureOptions, LEVEL_ANTIALIASING_MODES } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { describeRenderAntialiasing } from "@/core/render/lib/features/render-feature-choices";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

const MODE_OPTIONS = LEVEL_ANTIALIASING_MODES.map((value: ERendererAntialiasing) => ({
  label: describeRenderAntialiasing(value),
  value,
}));

interface ILevelAntialiasingActionProps extends BaseComponentProps {
  isOn: boolean;
  /** The mode the settings smooth with, which this view can only narrow to none. */
  settingsMode: ERendererAntialiasing;
  features: ILevelFeatureOptions;
  onToggle: () => void;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * Whether this view's edges are smoothed, and by which pass.
 */
export function LevelAntialiasingAction({
  "data-testid": dataTestId = "level-antialiasing-action",
  id,
  className,
  isOn,
  settingsMode,
  features,
  onToggle,
  onChange,
}: ILevelAntialiasingActionProps): ReactElement {
  const isAvailable: boolean = settingsMode !== ERendererAntialiasing.NONE;
  const mode: ERendererAntialiasing = features.antialiasing ?? settingsMode;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Antialiasing"}
      description={
        !isAvailable
          ? "Antialiasing is off in Settings, under Rendering"
          : isOn
            ? `Edges smoothed by ${describeRenderAntialiasing(mode)}`
            : "Antialiasing off, every edge as drawn"
      }
      icon={<DeblurIcon />}
      isOn={isOn && isAvailable}
      isDisabled={!isAvailable}
      toggleLabel={"Smooth the frame's edges"}
      onToggle={onToggle}
    >
      <RenderValueChoice
        label={"Mode"}
        options={MODE_OPTIONS}
        value={mode}
        onChange={(antialiasing: ERendererAntialiasing) => onChange({ ...features, antialiasing })}
      />

      <Button size={"small"} onClick={() => onChange({ ...features, antialiasing: null })}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
