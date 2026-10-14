import { formatBytes, formatCount } from "@xrf/format";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { RenderLoadReport, RenderMemoryReport } from "@/core/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelRendererResidentSectionProps extends BaseComponentProps {
  /** How far the renderer has read the level, or null until it says. */
  load: Nullable<RenderLoadReport>;
  /** What it holds on the GPU. */
  memory: RenderMemoryReport;
}

/**
 * @param memory - What the renderer holds on the GPU.
 * @returns What the GPU allocator holds allocated of what it reserved, or that it reports none, as a build or a backend
 *   reporting no allocator leaves it.
 */
function formatAllocated(memory: RenderMemoryReport): string {
  const allocated: unknown = memory.allocated;
  const reserved: unknown = memory.reserved;

  if (typeof allocated !== "number" || !Number.isFinite(allocated)) {
    return "Not reported";
  }

  return typeof reserved === "number" && Number.isFinite(reserved)
    ? `${formatBytes(allocated)} of ${formatBytes(reserved)} reserved`
    : formatBytes(allocated);
}

/**
 * What the level holds now: its sectors and textures, the geometry read for them, and what the GPU holds, of it and
 * of everything the renderer made.
 */
export function LevelRendererResidentSection({
  "data-testid": dataTestId = "level-renderer-resident-section",
  id,
  className,
  load,
  memory,
}: ILevelRendererResidentSectionProps): ReactElement {
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
      <EditorPanelProperty label={"GPU memory"} value={formatAllocated(memory)} />
    </EditorPanelSection>
  );
}
