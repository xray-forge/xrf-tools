import { Box, IconButton, Tooltip } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { CONTROL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** How far a toggle that is off is faded, which is the whole of the on/off vocabulary in this toolbar. */
const OFF_OPACITY: number = 0.45;

export interface IEditorViewToggleProps extends BaseComponentProps {
  /** Stable accessible name, independent of whether the toggle is on. */
  label: string;
  /** Explains the current state in the tooltip; defaults to the stable toggle label. */
  description?: string;
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
  "data-testid": dataTestId = "editor-view-toggle",
  id,
  className,
  label,
  description = label,
  icon,
  isOn,
  unavailableTitle,
  isDisabled = false,
  onToggle,
}: IEditorViewToggleProps): ReactElement {
  const hint: string = isDisabled && unavailableTitle ? unavailableTitle : description;

  return (
    <Tooltip title={hint} describeChild>
      <Box component={"span"} sx={{ display: "inline-flex" }}>
        <IconButton
          data-testid={dataTestId}
          id={id}
          className={className}
          aria-label={label}
          aria-description={hint}
          aria-pressed={isOn}
          color={"inherit"}
          disabled={isDisabled}
          size={"small"}
          sx={{
            opacity: isOn ? 1 : OFF_OPACITY,
            width: CONTROL.editorActionSize,
            height: CONTROL.editorActionSize,
            padding: 0,
            "& .MuiSvgIcon-root": { fontSize: CONTROL.editorActionIconSize },
          }}
          onClick={onToggle}
        >
          {icon}
        </IconButton>
      </Box>
    </Tooltip>
  );
}
