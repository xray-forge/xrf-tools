import { clamp } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { PANEL } from "@xrf/ui/theme";
import { useCallback } from "react";

import { TEditorPanelSide } from "@/core/shell/editor-shell";
import { clampPanelWidth } from "@/core/shell/panel/panel-width";
import { getPanelWidthStorageKey, IStoredValueCodec, textCodec, useStoredValue } from "@/core/storage";
import { useWindowWidth } from "@/lib/react/use-window-width";

const PREFERRED_WIDTH_CODEC: IStoredValueCodec<number> = textCodec(readPreferredWidth);

/**
 * Reads the stored preference, which is validated against the fixed bounds only - the window budget is a property of
 * the current window, not of what the user asked for.
 */
function readPreferredWidth(stored: Nullable<string>): number {
  const parsed: number = stored === null ? NaN : Number(stored);

  return Number.isFinite(parsed) ? clamp(parsed, PANEL.minWidth, PANEL.maxWidth) : PANEL.defaultWidth;
}

export interface IPanelWidth {
  width: number;
  onResize: (width: number) => void;
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

  const [preferred, setPreferred] = useStoredValue<number>(getPanelWidthStorageKey(side), PREFERRED_WIDTH_CODEC);

  const onResize = useCallback(
    (next: number) => setPreferred(clampPanelWidth(next, windowWidth, openCount)),
    [openCount, setPreferred, windowWidth]
  );

  return { onResize, width: clampPanelWidth(preferred, windowWidth, openCount) };
}
