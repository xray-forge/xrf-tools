import { Button, buttonClasses, Tooltip } from "@mui/material";
import { MouseEventHandler, ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorTextActionProps extends BaseComponentProps {
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
}: IEditorTextActionProps): ReactElement {
  return (
    <Tooltip describeChild title={description}>
      <span className={"inline-flex"}>
        <Button
          data-testid={dataTestId}
          id={id}
          className={cn("h-editor-action min-w-0 px-2 text-xs leading-none", className)}
          size={"small"}
          variant={variant}
          disabled={isDisabled}
          startIcon={icon}
          sx={{ [`& .${buttonClasses.startIcon}`]: { mr: 0.5 } }}
          onClick={onClick}
        >
          {label}
        </Button>
      </span>
    </Tooltip>
  );
}
