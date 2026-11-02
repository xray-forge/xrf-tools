import { Nullable } from "@xrf/types";
import { useCallback } from "react";

import { TShellContributionTool } from "@/core/shell/contributions/shell-contributions";
import { IShellPanel, TShellPanelSide } from "@/core/shell/panels/shell-panel";
import { getPanelSelectionStorageKey, IStoredValueCodec, textCodec, useStoredValue } from "@/core/storage";

/** The panel last opened, kept verbatim: an empty id for one closed by hand, or null where none was chosen. */
const SELECTION_CODEC: IStoredValueCodec<Nullable<string>> = textCodec(
  (stored: Nullable<string>) => stored,
  (id: Nullable<string>) => id
);

export interface IPanelSelection {
  activePanel: Nullable<IShellPanel>;
  activePanelId: Nullable<string>;
  onOpenPanel: (id: string) => void;
  onTogglePanel: (id: string) => void;
}

/**
 * Manages which panel is open on one side of the frame, remembered for each tool apart, so one tool's choice never
 * hides another's default.
 *
 * @param side - Frame side managed by the selection.
 * @param panels - Panels available on that side.
 * @param tool - The tool the choice is remembered for.
 * @returns The resolved panel and its toggle.
 */
export function usePanelSelection(
  side: TShellPanelSide,
  panels: Array<IShellPanel>,
  tool: TShellContributionTool
): IPanelSelection {
  const [activeId, onOpenPanel] = useStoredValue<Nullable<string>>(
    getPanelSelectionStorageKey(side, tool),
    SELECTION_CODEC
  );

  const selectedId: Nullable<string> = activeId ?? panels.find((panel) => panel.isOpenByDefault !== false)?.id ?? null;
  const activePanel: Nullable<IShellPanel> = panels.find((panel) => panel.id === selectedId) ?? null;
  const activePanelId: Nullable<string> = activePanel?.id ?? null;

  const onTogglePanel = useCallback(
    (id: string) => onOpenPanel(activePanelId === id ? "" : id),
    [activePanelId, onOpenPanel]
  );

  return { activePanel, activePanelId, onOpenPanel, onTogglePanel };
}
