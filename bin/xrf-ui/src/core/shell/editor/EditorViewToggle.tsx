import { IconButton, Tooltip } from "@mui/material";
import { ReactElement, ReactNode } from "react";

/** How far a toggle that is off is faded, which is the whole of the on/off vocabulary in this toolbar. */
const OFF_OPACITY: number = 0.45;

export interface IEditorViewToggleProps {
  /** Names the toggle in the tooltip and to a screen reader, which are the same word on purpose. */
  label: string;
  icon: ReactNode;
  isOn: boolean;
  /** Why the toggle is unavailable, shown instead of the label. Absent for a toggle that is always offered. */
  unavailableTitle?: string;
  isDisabled?: boolean;
  onToggle: () => void;
}

/**
 * One on/off view toggle of an editor toolbar.
 *
 * Lives with the toolbar rather than with any one editor, because the vocabulary is what makes two tools feel like
 * one program: a person who has learned that a faded icon means the view option is off has learned it everywhere.
 */
export function EditorViewToggle({
  label,
  icon,
  isOn,
  unavailableTitle,
  isDisabled = false,
  onToggle,
}: IEditorViewToggleProps): ReactElement {
  return (
    <Tooltip title={isDisabled && unavailableTitle ? unavailableTitle : label} describeChild>
      <span>
        <IconButton
          aria-label={label}
          color={"inherit"}
          disabled={isDisabled}
          sx={{ opacity: isOn ? 1 : OFF_OPACITY }}
          onClick={onToggle}
        >
          {icon}
        </IconButton>
      </span>
    </Tooltip>
  );
}
