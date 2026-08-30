import { useCallback, useState } from "react";

import { TEditorPanelSide } from "@/core/shell/panel/context";
import { clampPanelWidth } from "@/core/shell/panel/panel-width";
import { PANEL } from "@/core/theme/tokens";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { useWindowWidth } from "@/lib/react";
import { Nullable } from "@/lib/types/general";

export interface IPanelWidth {
  width: number;
  onResize: (width: number) => void;
}

/**
 * Reads the stored preference, which is validated against the fixed bounds only - the window budget is
 * a property of the current window, not of what the user asked for.
 */
function readPreferredWidth(side: TEditorPanelSide): number {
  const stored: Nullable<string> = getLocalStorageValue(`xrf.panels.${side}.width`);
  const parsed: number = stored === null ? NaN : Number(stored);

  return Number.isFinite(parsed) ? Math.min(PANEL.maxWidth, Math.max(PANEL.minWidth, parsed)) : PANEL.defaultWidth;
}

/**
 * Manages the width of the panel on one side of the frame.
 *
 * @param side - Frame side managed by the width.
 * @param openCount - Number of panels sharing the window budget.
 * @returns The width to render and the resize callback.
 */
export function usePanelWidth(side: TEditorPanelSide, openCount: number): IPanelWidth {
  const windowWidth: number = useWindowWidth();

  const [preferred, setPreferred] = useState<number>(() => readPreferredWidth(side));

  const onResize = useCallback(
    (next: number) => {
      const clamped: number = clampPanelWidth(next, windowWidth, openCount);

      setPreferred(clamped);
      setLocalStorageValue(`xrf.panels.${side}.width`, String(clamped));
    },
    [openCount, side, windowWidth]
  );

  return { onResize, width: clampPanelWidth(preferred, windowWidth, openCount) };
}
