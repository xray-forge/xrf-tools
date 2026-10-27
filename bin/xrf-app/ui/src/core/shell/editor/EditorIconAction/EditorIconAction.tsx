import { StyledComponentProps } from "@xrf/ui/element-types";
import { IconButton, svgIconClasses, Tooltip } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme/class-name";
import { getControlStateSx } from "@xrf/ui/theme/control-state";
import { mergeSx } from "@xrf/ui/theme/merge-sx";
import { CONTROL } from "@xrf/ui/theme/tokens";
import { AriaAttributes, MouseEventHandler, ReactElement, ReactNode } from "react";

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
  /** The secondary action, from a right click or the context menu key; the browser's own menu is the caller's. */
  onContextMenu?: MouseEventHandler<HTMLButtonElement>;
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
  onContextMenu,
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
          onContextMenu={onContextMenu}
        >
          {icon}
        </IconButton>
      </span>
    </Tooltip>
  );
}
