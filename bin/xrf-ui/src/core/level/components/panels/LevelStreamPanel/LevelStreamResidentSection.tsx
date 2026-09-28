import { ReactElement } from "react";

import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatCount } from "@/lib/format/number";
import { formatBytes } from "@/lib/memory/format";

interface ILevelStreamResidentSectionProps extends BaseComponentProps {
  stats: ILevelStats;
}

/**
 * What the level holds now: its sectors, and the geometry they cost.
 */
export function LevelStreamResidentSection({
  "data-testid": dataTestId = "level-stream-resident-section",
  id,
  className,
  stats,
}: ILevelStreamResidentSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Resident"} isFirst>
      <EditorPanelProperty label={"Sectors"} value={formatCount(stats.sectors)} />
      <EditorPanelProperty label={"Geometry"} value={formatBytes(stats.bytes)} />
    </EditorPanelSection>
  );
}
