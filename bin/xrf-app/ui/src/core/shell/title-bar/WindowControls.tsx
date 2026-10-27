import { Nullable } from "@xrf/types";
import { CloseGlyph } from "@xrf/ui/shell/title-bar/glyphs/CloseGlyph";
import { MaximizeGlyph } from "@xrf/ui/shell/title-bar/glyphs/MaximizeGlyph";
import { MinimizeGlyph } from "@xrf/ui/shell/title-bar/glyphs/MinimizeGlyph";
import { RestoreGlyph } from "@xrf/ui/shell/title-bar/glyphs/RestoreGlyph";
import { WindowControlButton } from "@xrf/ui/shell/title-bar/WindowControlButton";
import { ReactElement } from "react";

import { IWindowControls, useWindowControls } from "@/lib/tauri/use-window-controls";

/**
 * The minimize, maximize and close group, in the order the system puts them.
 *
 * Owns the window handle so nothing above it has to know whether one exists. Outside a tauri webview
 * the group renders nothing at all rather than rendering inert buttons, because a close button that
 * does nothing is worse than no close button.
 */
export function WindowControls(): Nullable<ReactElement> {
  const { isAvailable, isMaximized, minimize, toggleMaximize, close }: IWindowControls = useWindowControls();

  return isAvailable ? (
    <div className={"flex shrink-0 items-center"}>
      <WindowControlButton label={"Minimize"} onClick={minimize}>
        <MinimizeGlyph />
      </WindowControlButton>

      <WindowControlButton label={isMaximized ? "Restore down" : "Maximize"} onClick={toggleMaximize}>
        {isMaximized ? <RestoreGlyph /> : <MaximizeGlyph />}
      </WindowControlButton>

      <WindowControlButton isDanger={true} label={"Close"} onClick={close}>
        <CloseGlyph />
      </WindowControlButton>
    </div>
  ) : null;
}
