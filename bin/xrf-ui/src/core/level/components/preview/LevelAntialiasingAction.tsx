import { default as DeblurIcon } from "@mui/icons-material/Deblur";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { ERenderAntialiasing, RenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import { describeLevelFeatureToggle, ILevelFeatureOptions, LEVEL_ANTIALIASING_MODES } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { describeRenderAntialiasing, IRenderChoiceOption } from "@/core/render/lib/features";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

const MODE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderAntialiasing>> = LEVEL_ANTIALIASING_MODES.map(
  (value: RenderAntialiasing) => ({ label: describeRenderAntialiasing(value), value })
);

interface ILevelAntialiasingActionProps extends BaseComponentProps {
  isOn: boolean;
  /** The mode the settings smooth with, which this view can only narrow to none. */
  settingsMode: RenderAntialiasing;
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
  const isAvailable: boolean = settingsMode !== ERenderAntialiasing.NONE;
  const mode: RenderAntialiasing = features.antialiasing ?? settingsMode;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Antialiasing"}
      description={describeLevelFeatureToggle({
        isAvailable,
        isOn,
        label: "Antialiasing",
        off: "Antialiasing off, every edge as drawn",
        on: `Edges smoothed by ${describeRenderAntialiasing(mode)}`,
      })}
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
        onChange={(antialiasing: RenderAntialiasing) => onChange({ ...features, antialiasing })}
      />

      <Button size={"small"} onClick={() => onChange({ ...features, antialiasing: null })}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
