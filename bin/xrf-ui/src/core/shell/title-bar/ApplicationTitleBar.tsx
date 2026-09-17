import { LinearProgress } from "@mui/material";
import { ReactElement, Ref } from "react";

import { ApplicationTitleBarIcon } from "@/core/shell/title-bar/ApplicationTitleBarIcon";
import { WindowControls } from "@/core/shell/title-bar/WindowControls";
import { DELAYED_REVEAL_SX } from "@/core/ui/layout/delayed-reveal";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IApplicationTitleBarProps extends BaseComponentProps {
  /** Where the active application portals its toolbar. The frame owns the element. */
  toolbarRef?: Ref<HTMLDivElement>;
  /** Draws the band's own progress line. Every editor already publishes this through `useEditorBusy`. */
  isBusy?: boolean;
}

/**
 * The window's single top band: drawn caption and the active application's toolbar in one row.
 */
export function ApplicationTitleBar({
  "data-testid": dataTestId = "application-title-bar",
  id = "application-title-bar",
  className,
  toolbarRef,
  isBusy,
}: IApplicationTitleBarProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      data-tauri-drag-region={"deep"}
      id={id}
      className={cn("relative flex h-title-bar min-h-title-bar shrink-0 items-center select-none", className)}
    >
      <ApplicationTitleBarIcon />

      <div ref={toolbarRef} className={"flex h-full min-w-0 grow items-center"} />

      <WindowControls />

      {isBusy ? <LinearProgress className={"absolute right-0 bottom-0 left-0 h-0.5"} sx={DELAYED_REVEAL_SX} /> : null}
    </div>
  );
}
