import { Box, IconButton, Tooltip } from "@mui/material";
import { MouseEventHandler, ReactElement, ReactNode } from "react";

import { CONTROL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorIconActionProps extends BaseComponentProps {
  label: string;
  description: string;
  icon: ReactNode;
  isDisabled?: boolean;
  onClick: MouseEventHandler<HTMLButtonElement>;
}

/**
 * Compact editor action with an accessible name and a tooltip that remains available while disabled.
 */
export function EditorIconAction({
  "data-testid": dataTestId = "editor-icon-action",
  id,
  className,
  label,
  description,
  icon,
  isDisabled = false,
  onClick,
}: IEditorIconActionProps): ReactElement {
  return (
    <Tooltip describeChild title={description}>
      <Box component={"span"} sx={{ display: "inline-flex" }}>
        <IconButton
          data-testid={dataTestId}
          id={id}
          className={className}
          aria-label={label}
          aria-description={description}
          color={"inherit"}
          disabled={isDisabled}
          size={"small"}
          sx={{
            width: CONTROL.editorActionSize,
            height: CONTROL.editorActionSize,
            padding: 0,
            "& .MuiSvgIcon-root": { fontSize: CONTROL.editorActionIconSize },
          }}
          onClick={onClick}
        >
          {icon}
        </IconButton>
      </Box>
    </Tooltip>
  );
}
