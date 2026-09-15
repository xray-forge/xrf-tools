import { CommandType, QueryType } from "@wirestate/core";

import { TEditorPanelSide } from "@/core/shell/editor-shell";

/** Names one side's panel, whether opening it or asking after it. */
export interface IPanelSideMessage {
  side: TEditorPanelSide;
}

/** Names the panel to show on one side. */
export interface IPanelSetActiveMessage extends IPanelSideMessage {
  panelId: string;
}

/** Shows a panel, whatever was open on that side. */
export const PANEL_SET_ACTIVE_MESSAGE: CommandType = Symbol("@/panel/set-active");

/** Answers which panel a side currently shows, or null when the side is closed. */
export const PANEL_ACTIVE_QUERY: QueryType = Symbol("@/panel/active");
