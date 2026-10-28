import { ReactElement, ReactNode } from "react";

import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { IFrameSide } from "@/core/shell/panels/use-frame-sides";

import { PanelStripeButton } from "./rail";

interface IApplicationPanelStripeProps {
  frameSide: IFrameSide;
  /** The shell's controls that open no panel, below its panels. */
  footer?: ReactNode;
}

/**
 * One edge of the window frame: the tool's panels at the top, the shell's own panels and controls at the
 * bottom.
 *
 * Both sides render through here so they cannot drift apart. The stripe stays put even when a
 * tool declares no panels, for the same reason every route has a toolbar - a frame that changes
 * shape as you move between tools is harder to read than one that does not.
 */
export function ApplicationPanelStripe({
  frameSide: { side, toolPanels, shellPanels, activePanelId, onTogglePanel },
  footer,
}: IApplicationPanelStripeProps): ReactElement {
  return (
    <div className={"flex w-rail min-w-rail flex-col items-center gap-1.5 py-2"}>
      {toolPanels.map((panel: IShellPanel) => (
        <PanelStripeButton
          key={panel.id}
          panel={panel}
          side={side}
          isActive={panel.id === activePanelId}
          onTogglePanel={onTogglePanel}
        />
      ))}

      <div className={"grow"} />

      {shellPanels.map((panel: IShellPanel) => (
        <PanelStripeButton
          key={panel.id}
          panel={panel}
          side={side}
          isActive={panel.id === activePanelId}
          isShellOwned={true}
          onTogglePanel={onTogglePanel}
        />
      ))}

      {footer}
    </div>
  );
}
