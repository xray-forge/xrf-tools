import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

export interface ILevelSettingsResetButtonProps extends BaseComponentProps {
  /** What the popover holds more than one of names the one it sets back, as a screen reader hears it. */
  subject?: string;
  onReset: () => void;
}

/** Clears what a popover set for the view, which then follows Settings › Rendering again. */
export function LevelSettingsResetButton({
  "data-testid": dataTestId = "level-settings-reset-button",
  id,
  className,
  subject,
  onReset,
}: ILevelSettingsResetButtonProps): ReactElement {
  return (
    <Button
      data-testid={dataTestId}
      id={id}
      className={className}
      size={"small"}
      aria-label={subject ? `Set the ${subject} back to the settings` : undefined}
      onClick={onReset}
    >
      Back to the settings
    </Button>
  );
}
