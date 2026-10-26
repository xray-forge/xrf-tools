import { CommandType, QueryType } from "@wirestate/core";

import { TEditorPanelSide } from "@/core/shell/editor-shell";

/** Names one side's panel, whether opening it or asking after it. */
export interface IPanelSideCommand {
  side: TEditorPanelSide;
}

/** Names the panel to show on one side. */
export interface IPanelSetActiveCommand extends IPanelSideCommand {
  panelId: string;
}

/** Shows a panel, whatever was open on that side. */
export const PANEL_SET_ACTIVE_COMMAND: CommandType = Symbol("@/panel/set-active");

/** Closes whatever one side has open. */
export const PANEL_CLOSE_COMMAND: CommandType = Symbol("@/panel/close");

/** Answers which panel a side currently shows, or null when the side is closed. */
export const PANEL_ACTIVE_QUERY: QueryType = Symbol("@/panel/active");
