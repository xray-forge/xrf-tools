import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { PanelResizer } from "@/core/shell/panels/PanelResizer";
import { IFrameSide } from "@/core/shell/panels/use-frame-sides";

interface IApplicationPanelSlotProps {
  frameSide: IFrameSide;
}

/**
 * The docked panel on one side of the content, and the handle that sizes it.
 *
 * Two boxes rather than one: the scrolling half has to be a separate child, or the resizer - absolutely
 * positioned to cost no width - would scroll away with the panel's content.
 */
export function ApplicationPanelSlot({
  frameSide: { side, activePanel, width, onResize },
}: IApplicationPanelSlotProps): ReactElement {
  if (!activePanel) {
    return <></>;
  }

  return (
    <div
      data-testid={`application-panel-slot-${side}`}
      data-panel-side={side}
      className={cn(
        "relative flex min-h-0 flex-col border-divider surface-frame",
        side === "left" ? "border-r" : "border-l"
      )}
      // Measured rather than authored: the slot's width is whatever the drag and the window budget left it.
      style={{ minWidth: width, width }}
    >
      <div className={"min-h-0 grow overflow-y-auto"}>{activePanel.render()}</div>

      <PanelResizer side={side} width={width} onResize={onResize} />
    </div>
  );
}
