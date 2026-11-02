import { BaseComponentProps } from "@xrf/ui/element-types";
import { LinearProgress } from "@xrf/ui/material/progress";
import { cn } from "@xrf/ui/theme";
import { ReactElement, Ref } from "react";

import { ApplicationMenu } from "@/core/shell/title-bar/ApplicationMenu";
import { ApplicationTitleBarIcon } from "@/core/shell/title-bar/ApplicationTitleBarIcon";
import { WindowControls } from "@/core/shell/title-bar/WindowControls";

interface IApplicationTitleBarProps extends BaseComponentProps {
  /** Where the active tool portals its toolbar. The frame owns the element. */
  toolbarRef?: Ref<HTMLDivElement>;
  /** Draws the band's own progress line. Every tool already publishes this through `useToolBusy`. */
  isBusy?: boolean;
}

/**
 * The window's single top band: drawn caption, the window's menu and the active tool's toolbar in one row.
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
      className={cn(
        "group/caption relative flex h-title-bar min-h-title-bar shrink-0 items-center select-none",
        className
      )}
    >
      <ApplicationTitleBarIcon />

      <ApplicationMenu />

      <div ref={toolbarRef} className={"flex h-full min-w-0 grow items-center"} />

      <WindowControls />

      {isBusy ? (
        <LinearProgress className={"invisible absolute right-0 bottom-0 left-0 h-0.5 animate-delayed-reveal"} />
      ) : null}
    </div>
  );
}
