import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

export interface ILevelSettingsResetButtonProps extends BaseComponentProps {
  /** What the popover holds more than one of names the one it sets back, as a screen reader hears it. */
  subject?: string;
  onReset: () => void;
}

/** What a reset of the view's own values says it does: the view follows Settings › Rendering again. */
export const LEVEL_SETTINGS_RESET_LABEL: string = "Back to the settings";

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
      {LEVEL_SETTINGS_RESET_LABEL}
    </Button>
  );
}
