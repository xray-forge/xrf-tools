import { IconButton, svgIconClasses, Tooltip } from "@mui/material";
import { AriaAttributes, MouseEventHandler, ReactElement, ReactNode } from "react";

import { getControlStateSx } from "@/core/theme/control-state";
import { mergeSx } from "@/core/theme/merge-sx";
import { CONTROL } from "@/core/theme/tokens";
import { cn } from "@/lib/dom/dom-name";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface IEditorIconActionProps
  extends
    StyledComponentProps,
    Pick<AriaAttributes, "aria-controls" | "aria-expanded" | "aria-haspopup" | "aria-pressed"> {
  label: string;
  description: string;
  icon: ReactNode;
  isDisabled?: boolean;
  /** The control is doing something: a toggle that is on, or a value that differs from its default. */
  isActive?: boolean;
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
  isActive = false,
  onClick,
  sx,
  ...rest
}: IEditorIconActionProps): ReactElement {
  return (
    <Tooltip describeChild title={description}>
      <span className={"inline-flex"}>
        <IconButton
          {...rest}
          data-testid={dataTestId}
          aria-label={label}
          aria-description={description}
          id={id}
          className={cn("h-editor-action w-editor-action p-0", className)}
          disabled={isDisabled}
          size={"small"}
          sx={mergeSx(
            getControlStateSx(isActive),
            { [`& .${svgIconClasses.root}`]: { fontSize: CONTROL.editorActionIconSize } },
            sx
          )}
          onClick={onClick}
        >
          {icon}
        </IconButton>
      </span>
    </Tooltip>
  );
}
