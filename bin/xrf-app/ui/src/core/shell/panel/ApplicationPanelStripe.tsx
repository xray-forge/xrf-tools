import { EMPTY_ARRAY } from "@xrf/collections";
import { Nullable } from "@xrf/types";
import { RailButton } from "@xrf/ui/shell/rail-button";
import { ReactElement, ReactNode } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/editor-shell";

import { PanelStripeButton } from "./rail";

interface IApplicationPanelStripeProps {
  side: TEditorPanelSide;
  /** The open tool's panels, at the top. */
  panels: ReadonlyArray<IEditorPanel>;
  /** The shell's own panels, pinned to the bottom with its other controls. */
  shellPanels?: ReadonlyArray<IEditorPanel>;
  activePanelId: Nullable<string>;
  /** The shell's controls that open no panel, below its panels. */
  footer?: ReactNode;
  onTogglePanel: (id: string) => void;
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
  side,
  panels,
  shellPanels = EMPTY_ARRAY,
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

      {/* Drawn as the shell's controls are, in the rail's gradient, so what the shell owns reads apart. */}
      {shellPanels.map((panel: IEditorPanel) => (
        <RailButton
          key={panel.id}
          label={panel.label}
          icon={panel.icon}
          tooltipPlacement={side === "left" ? "right" : "left"}
          isSelected={panel.id === activePanelId}
          onClick={() => onTogglePanel(panel.id)}
        />
      ))}

      {footer}
    </div>
  );
}
