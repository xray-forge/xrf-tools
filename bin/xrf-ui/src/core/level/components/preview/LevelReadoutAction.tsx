import { default as QueryStatsIcon } from "@mui/icons-material/QueryStats";
import { ReactElement } from "react";

import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelReadoutActionProps extends BaseComponentProps {
  isOn: boolean;
  /** Whether the frame readout lists each pass's GPU time. */
  isAdvanced: boolean;
  onToggle: () => void;
  onToggleAdvanced: () => void;
}

/**
 * Whether the readouts are laid over the viewport, and whether the frame's lists what each pass cost.
 */
export function LevelReadoutAction({
  "data-testid": dataTestId = "level-readout-action",
  id,
  className,
  isOn,
  isAdvanced,
  onToggle,
  onToggleAdvanced,
}: ILevelReadoutActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Readout"}
      description={
        isOn
          ? isAdvanced
            ? "Frame cost with each pass's GPU time, and the camera's place"
            : "Frame cost and the camera's place"
          : "Readouts off, a clean look at the level"
      }
      icon={<QueryStatsIcon />}
      isOn={isOn}
      toggleLabel={"Show the readouts"}
      onToggle={onToggle}
    >
      <CheckboxFormRow label={"Advanced stats: GPU time per pass"} isChecked={isAdvanced} onChange={onToggleAdvanced} />
    </EditorPopoverToggle>
  );
}
