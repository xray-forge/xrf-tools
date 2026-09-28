import { default as QueryStatsIcon } from "@mui/icons-material/QueryStats";
import { ReactElement } from "react";

import { RenderPassTimingFormRow } from "@/core/render/components/controls/RenderPassTimingFormRow";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelReadoutActionProps extends BaseComponentProps {
  isOn: boolean;
  /** Whether the settings time every pass on the GPU, which the frame readout then lists. */
  isGpuTimed: boolean;
  onToggle: () => void;
  /** Sets in the settings whether every viewport times its passes. */
  onChangeGpuTimed: (isGpuTimed: boolean) => void;
}

/**
 * Whether the readouts are laid over the viewport, and whether every pass is timed for the frame's.
 */
export function LevelReadoutAction({
  "data-testid": dataTestId = "level-readout-action",
  id,
  className,
  isOn,
  isGpuTimed,
  onToggle,
  onChangeGpuTimed,
}: ILevelReadoutActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Readout"}
      description={
        isOn
          ? isGpuTimed
            ? "Frame cost with each pass's GPU time, and the camera's place"
            : "Frame cost and the camera's place"
          : "Readouts off, a clean look at the level"
      }
      icon={<QueryStatsIcon />}
      isOn={isOn}
      toggleLabel={"Show the readouts"}
      onToggle={onToggle}
    >
      <RenderPassTimingFormRow isChecked={isGpuTimed} onChange={onChangeGpuTimed} />
    </EditorPopoverToggle>
  );
}
