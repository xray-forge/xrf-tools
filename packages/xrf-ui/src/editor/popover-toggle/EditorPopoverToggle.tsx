import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback } from "react";

import { EditorPopoverAction } from "#/editor/popover-action";
import { BaseComponentProps } from "#/element-types";
import { CheckboxFormRow } from "#/form/checkbox-row";
import { Typography } from "#/material/typography";

export interface IEditorPopoverToggleProps extends BaseComponentProps {
  /** Stable accessible name, independent of whether the toggle is on. */
  label: string;
  /** What the settings behind it are at now, said in the tooltip ahead of the right click hint while enabled. */
  description: string;
  icon: ReactNode;
  isOn: boolean;
  /** Names the checkbox at the head of the popover, which is the toggle for a keyboard. */
  toggleLabel: string;
  /** The toggle cannot turn over: a click opens its settings, where `description` says why in their place. */
  isDisabled?: boolean;
  /** Whether its settings take a wider popover, for rows of several named choices. */
  isWide?: boolean;
  /** What the popover sets ahead of the toggle, whether the toggle is on or not. */
  header?: ReactNode;
  /** Why the settings behind it do nothing now, said in their place; the toggle still turns over. */
  lockedReason?: Nullable<string>;
  onToggle: () => void;
  /** The settings the toggle turns on and off. */
  children: ReactNode;
}

/**
 * A view toggle with settings behind it: a click turns it over, a right click opens them; a click opens them too while
 * it cannot turn over.
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
  isWide = false,
  header = null,
  lockedReason = null,
  onToggle,
  children,
}: IEditorPopoverToggleProps): ReactElement {
  const reason: Nullable<string> = isDisabled ? description : lockedReason;

  const onClick = useCallback(() => onToggle(), [onToggle]);

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={isDisabled ? description : `${description}. Right-click for its settings`}
      icon={icon}
      isActive={isOn && !isDisabled}
      // A click opens it while it cannot turn over, which its cue then says.
      kind={isDisabled ? "popover" : "toggle"}
      onClick={isDisabled ? undefined : onClick}
    >
      <div className={`flex ${isWide ? "w-80" : "w-60"} flex-col gap-2 px-4 py-2`}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          {label}
        </Typography>

        {header}

        <CheckboxFormRow
          label={toggleLabel}
          isChecked={isOn && !isDisabled}
          isDisabled={isDisabled}
          onChange={onToggle}
        />

        {reason ? (
          <Typography className={"text-text-secondary"} variant={"caption"}>
            {reason}
          </Typography>
        ) : (
          children
        )}
      </div>
    </EditorPopoverAction>
  );
}
