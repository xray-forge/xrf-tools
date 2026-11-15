import { ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme";

import { CloseGlyph } from "./CloseGlyph";
import { MaximizeGlyph } from "./MaximizeGlyph";
import { MinimizeGlyph } from "./MinimizeGlyph";
import { RestoreGlyph } from "./RestoreGlyph";
import { WindowControlButton } from "./WindowControlButton";

export interface IWindowControlsProps extends BaseComponentProps {
  /** Swaps maximize for restore, as the system does while the window fills the screen. */
  isMaximized: boolean;
  onMinimize: () => void;
  onToggleMaximize: () => void;
  onClose: () => void;
}

/**
 * The minimize, maximize and close group, in the order the system puts them.
 */
export function WindowControls({
  "data-testid": dataTestId = "window-controls",
  id,
  className,
  isMaximized,
  onMinimize,
  onToggleMaximize,
  onClose,
}: IWindowControlsProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex shrink-0 items-center", className)}>
      <WindowControlButton label={"Minimize"} onClick={onMinimize}>
        <MinimizeGlyph />
      </WindowControlButton>

      <WindowControlButton label={isMaximized ? "Restore down" : "Maximize"} onClick={onToggleMaximize}>
        {isMaximized ? <RestoreGlyph /> : <MaximizeGlyph />}
      </WindowControlButton>

      <WindowControlButton isDanger={true} label={"Close"} onClick={onClose}>
        <CloseGlyph />
      </WindowControlButton>
    </div>
  );
}
