import { ReactElement } from "react";

import { ILevelResidencyOptions } from "@/core/level/lib/residency/level-residency";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { formatBytes } from "@/lib/memory/format";

interface ILevelStreamBudgetSectionProps {
  /** Bytes of geometry held. */
  bytes: number;
  residency: ILevelResidencyOptions;
}

/**
 * What the level may hold at once and how far out, against what it holds.
 */
export function LevelStreamBudgetSection({ bytes, residency }: ILevelStreamBudgetSectionProps): ReactElement {
  return (
    <EditorPanelSection title={"Budget"}>
      <EditorPanelProperty label={"Memory"} value={`${formatBytes(bytes)} of ${formatBytes(residency.memoryBudget)}`} />
      <EditorPanelProperty label={"Sector cap"} value={residency.maxSectors} />
      <EditorPanelProperty label={"Reads at once"} value={residency.concurrency} />
      <EditorPanelProperty label={"Fills the level in"} value={residency.isPreloaded ? "yes" : "no"} />
      <EditorPanelProperty label={"Load distance"} value={`${residency.loadDistance.toFixed(0)} m`} />
      <EditorPanelProperty label={"Keep distance"} value={`${residency.keepDistance.toFixed(0)} m`} />
    </EditorPanelSection>
  );
}
