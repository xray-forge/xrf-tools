import { Typography } from "@mui/material";
import { MouseEvent, ReactElement, ReactNode, useCallback } from "react";

import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IEditorPopoverToggleProps extends BaseComponentProps {
  /** Stable accessible name, independent of whether the toggle is on. */
  label: string;
  /** What the settings behind it are at now, said in the tooltip ahead of the right click hint. */
  description: string;
  icon: ReactNode;
  isOn: boolean;
  /** Names the checkbox at the head of the popover, which is the toggle for a keyboard. */
  toggleLabel: string;
  isDisabled?: boolean;
  onToggle: () => void;
  /** The settings the toggle turns on and off. */
  children: ReactNode;
}

/**
 * A view toggle with settings behind it: a click opens them, a right click turns the toggle over.
 */
export function EditorPopoverToggle({
  "data-testid": dataTestId = "editor-popover-toggle",
  id,
  className,
  label,
  description,
  icon,
  isOn,
  toggleLabel,
  isDisabled = false,
  onToggle,
  children,
}: IEditorPopoverToggleProps): ReactElement {
  const onContextMenu = useCallback(
    (event: MouseEvent<HTMLButtonElement>) => {
      event.preventDefault();
      onToggle();
    },
    [onToggle]
  );

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={`${description}. Right-click to turn ${isOn ? "off" : "on"}`}
      icon={icon}
      isActive={isOn}
      isDisabled={isDisabled}
      onContextMenu={onContextMenu}
    >
      <div className={"flex w-60 flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          {label}
        </Typography>

        <CheckboxFormRow label={toggleLabel} isChecked={isOn} onChange={onToggle} />

        {children}
      </div>
    </EditorPopoverAction>
  );
}
