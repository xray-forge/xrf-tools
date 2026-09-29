import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IEditorPopoverGroupProps extends BaseComponentProps {
  /** Stable accessible name of the trigger and the popover. */
  label: string;
  /** What the toggles behind it are at now, said in the tooltip. */
  description: string;
  icon: ReactNode;
  /** Whether any toggle behind it is on, which is painted as a toggle that is on. */
  isActive: boolean;
  /** The toggles, each an `EditorPopoverGroupSection`. */
  children: ReactNode;
}

/**
 * A few related view toggles behind one trigger: a click opens them.
 */
export function EditorPopoverGroup({
  "data-testid": dataTestId = "editor-popover-group",
  id,
  className,
  label,
  description,
  icon,
  isActive,
  children,
}: IEditorPopoverGroupProps): ReactElement {
  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={description}
      icon={icon}
      isActive={isActive}
    >
      <div className={"flex w-64 flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          {label}
        </Typography>

        {children}
      </div>
    </EditorPopoverAction>
  );
}
