import { Box, IconButton, Tooltip } from "@mui/material";
import { AriaAttributes, MouseEventHandler, ReactElement, ReactNode } from "react";

import { mergeSx } from "@/core/theme/merge-sx";
import { CONTROL } from "@/core/theme/tokens";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface IEditorIconActionProps
  extends
    StyledComponentProps,
    Pick<AriaAttributes, "aria-controls" | "aria-expanded" | "aria-haspopup" | "aria-pressed"> {
  label: string;
  description: string;
  icon: ReactNode;
  isDisabled?: boolean;
  isHighlighted?: boolean;
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
  isHighlighted = false,
  onClick,
  sx,
  ...rest
}: IEditorIconActionProps): ReactElement {
  return (
    <Tooltip describeChild title={description}>
      <Box component={"span"} sx={{ display: "inline-flex" }}>
        <IconButton
          {...rest}
          data-testid={dataTestId}
          aria-label={label}
          aria-description={description}
          id={id}
          className={className}
          color={isHighlighted ? "primary" : "inherit"}
          disabled={isDisabled}
          size={"small"}
          sx={mergeSx(
            {
              width: CONTROL.editorActionSize,
              height: CONTROL.editorActionSize,
              padding: 0,
              "& .MuiSvgIcon-root": { fontSize: CONTROL.editorActionIconSize },
            },
            sx
          )}
          onClick={onClick}
        >
          {icon}
        </IconButton>
      </Box>
    </Tooltip>
  );
}
