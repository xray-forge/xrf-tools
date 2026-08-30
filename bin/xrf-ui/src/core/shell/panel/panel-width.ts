import { PANEL } from "@/core/theme/tokens";

/**
 * Resolves the width a panel actually renders at.
 *
 * @param preferred - Width the user asked for, which outlives any window too narrow to honour it.
 * @param windowWidth - Current viewport width.
 * @param openCount - Number of panels sharing the budget.
 * @returns Width to render, never above the budget share unless the floor forces it.
 */
export function clampPanelWidth(preferred: number, windowWidth: number, openCount: number): number {
  const share: number = (windowWidth * PANEL.maxWidthRatio) / Math.max(1, openCount);

  return Math.max(PANEL.minWidth, Math.min(PANEL.maxWidth, share, preferred));
}
