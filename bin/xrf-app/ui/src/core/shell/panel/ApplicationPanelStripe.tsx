import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/editor-shell";

import { PanelStripeButton } from "./rail";

interface IApplicationPanelStripeProps {
  side: TEditorPanelSide;
  panels: Array<IEditorPanel>;
  activePanelId: Nullable<string>;
  /** Pinned to the bottom: what the shell owns, below what the application declared. */
  footer?: ReactNode;
  onTogglePanel: (id: string) => void;
}

/**
 * One edge of the window frame: the application's panels at the top, the shell's own controls at the
 * bottom.
 *
 * Both sides render through here so they cannot drift apart. The stripe stays put even when an
 * application declares no panels, for the same reason every route has a toolbar - a frame that changes
 * shape as you move between applications is harder to read than one that does not.
 */
export function ApplicationPanelStripe({
  side,
  panels,
  activePanelId,
  footer,
  onTogglePanel,
}: IApplicationPanelStripeProps): ReactElement {
  return (
    <div className={"flex w-rail min-w-rail flex-col items-center gap-1.5 py-2"}>
      {panels.map((panel: IEditorPanel) => (
        <PanelStripeButton
          key={panel.id}
          panel={panel}
          side={side}
          isActive={panel.id === activePanelId}
          onTogglePanel={onTogglePanel}
        />
      ))}

      <div className={"grow"} />

      {footer}
    </div>
  );
}
