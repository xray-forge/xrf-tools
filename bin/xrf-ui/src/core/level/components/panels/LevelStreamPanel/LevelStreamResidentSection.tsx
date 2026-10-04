import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { RenderLoadReport, RenderMemoryReport } from "@/core/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatCount } from "@/lib/format/number";
import { formatBytes } from "@/lib/memory/format";

interface ILevelStreamResidentSectionProps extends BaseComponentProps {
  /** How far the renderer has read the level, or null until it says. */
  load: Nullable<RenderLoadReport>;
  /** What it holds on the GPU. */
  memory: RenderMemoryReport;
}

/**
 * What the level holds now: its sectors and textures, the geometry read for them, and what the GPU holds.
 */
export function LevelStreamResidentSection({
  "data-testid": dataTestId = "level-stream-resident-section",
  id,
  className,
  load,
  memory,
}: ILevelStreamResidentSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Resident"} isFirst>
      <EditorPanelProperty
        label={"Sectors"}
        value={`${formatCount(load?.sectors ?? 0)} of ${formatCount(load?.sectorsTotal ?? 0)}`}
      />
      <EditorPanelProperty
        label={"Textures"}
        value={`${formatCount(load?.textures ?? 0)} of ${formatCount(load?.texturesTotal ?? 0)}`}
      />
      <EditorPanelProperty label={"Geometry read"} value={formatBytes(load?.bytes ?? 0)} />
      <EditorPanelProperty label={"GPU scene buffers"} value={formatBytes(memory.scene)} />
      <EditorPanelProperty label={"GPU textures"} value={formatBytes(memory.textures)} />
    </EditorPanelSection>
  );
}
