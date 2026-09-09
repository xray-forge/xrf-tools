import { Box, Button, Tooltip } from "@mui/material";
import { MouseEventHandler, ReactElement, ReactNode } from "react";

import { mergeSx } from "@/core/theme/merge-sx";
import { CONTROL } from "@/core/theme/tokens";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface IEditorTextActionProps extends StyledComponentProps {
  label: string;
  description: string;
  icon?: ReactNode;
  isDisabled?: boolean;
  variant?: "contained" | "outlined" | "text";
  onClick: MouseEventHandler<HTMLButtonElement>;
}

/** Compact labeled editor action with a tooltip that also works while disabled. */
export function EditorTextAction({
  "data-testid": dataTestId,
  id,
  className,
  label,
  description,
  icon,
  isDisabled = false,
  variant = "outlined",
  onClick,
  sx,
}: IEditorTextActionProps): ReactElement {
  return (
    <Tooltip describeChild title={description}>
      <Box component={"span"} sx={{ display: "inline-flex" }}>
        <Button
          data-testid={dataTestId}
          id={id}
          className={className}
          size={"small"}
          variant={variant}
          disabled={isDisabled}
          startIcon={icon}
          sx={mergeSx(
            {
              height: CONTROL.editorActionSize,
              minWidth: 0,
              px: 1,
              fontSize: CONTROL.editorActionFontSize,
              lineHeight: 1,
              "& .MuiButton-startIcon": { mr: 0.5 },
            },
            sx
          )}
          onClick={onClick}
        >
          {label}
        </Button>
      </Box>
    </Tooltip>
  );
}
