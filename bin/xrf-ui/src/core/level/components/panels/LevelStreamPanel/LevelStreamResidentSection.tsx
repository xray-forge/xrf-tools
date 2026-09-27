import { ReactElement } from "react";

import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { formatCount } from "@/lib/format/number";
import { formatBytes } from "@/lib/memory/format";

interface ILevelStreamResidentSectionProps {
  stats: ILevelStats;
}

/**
 * What the level holds now: its sectors, and the geometry they cost.
 */
export function LevelStreamResidentSection({ stats }: ILevelStreamResidentSectionProps): ReactElement {
  return (
    <EditorPanelSection title={"Resident"} isFirst>
      <EditorPanelProperty label={"Sectors"} value={formatCount(stats.sectors)} />
      <EditorPanelProperty label={"Geometry"} value={formatBytes(stats.bytes)} />
    </EditorPanelSection>
  );
}
