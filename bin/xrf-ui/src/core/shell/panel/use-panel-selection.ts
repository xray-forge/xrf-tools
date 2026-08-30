import { useCallback, useEffect, useState } from "react";

import { IEditorPanel, TEditorPanelSide } from "@/core/shell/panel/context";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Nullable } from "@/lib/types/general";

export interface IPanelSelection {
  activePanel: Nullable<IEditorPanel>;
  activePanelId: Nullable<string>;
  onTogglePanel: (id: string) => void;
}

/**
 * Manages which panel is open on one side of the frame.
 *
 * @param side - Frame side managed by the selection.
 * @param panels - Panels available on that side.
 * @param selectionScope - Stable scope used to persist the active panel.
 * @returns The resolved panel and its toggle.
 */
export function usePanelSelection(
  side: TEditorPanelSide,
  panels: Array<IEditorPanel>,
  selectionScope: string
): IPanelSelection {
  const storageKey: string = `xrf.panels.${side}.${selectionScope}`;

  const [activeId, setActiveId] = useState<Nullable<string>>(null);

  const defaultPanelId: Nullable<string> = panels.find((panel) => panel.isOpenByDefault !== false)?.id ?? null;

  const resolvedPanelId: Nullable<string> =
    activeId === null ? defaultPanelId : panels.some((panel) => panel.id === activeId) ? activeId : null;

  const activePanel: Nullable<IEditorPanel> = panels.find((panel) => panel.id === resolvedPanelId) ?? null;

  const onTogglePanel = useCallback(
    (id: string) => {
      const next: string = resolvedPanelId === id ? "" : id;

      setActiveId(next);
      setLocalStorageValue(storageKey, next);
    },
    [resolvedPanelId, storageKey]
  );

  useEffect(() => {
    setActiveId(getLocalStorageValue(storageKey));
  }, [storageKey]);

  return { activePanel, activePanelId: resolvedPanelId, onTogglePanel };
}
