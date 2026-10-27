import { formatBytes, formatCount } from "@xrf/format";
import { RenderLoadReport, RenderMemoryReport } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditorPanelProperty } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelProperty";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement } from "react";

/**
 * @param bytes - A count of bytes the report may leave out.
 * @returns Whether it holds one.
 */
function isReported(bytes: unknown): bytes is number {
  return typeof bytes === "number" && Number.isFinite(bytes);
}

/**
 * @param memory - What the renderer holds on the GPU.
 * @returns What the GPU allocator holds allocated of what it reserved, or that it reports none, as a build or a backend
 *   reporting no allocator leaves it.
 */
function formatAllocated(memory: RenderMemoryReport): string {
  if (!isReported(memory.allocated)) {
    return "Not reported";
  }

  return isReported(memory.reserved)
    ? `${formatBytes(memory.allocated)} of ${formatBytes(memory.reserved)} reserved`
    : formatBytes(memory.allocated);
}

/**
 * @param memory - What the renderer holds on the GPU.
 * @returns The most the GPU allocator held allocated since the renderer started, or that it reports none.
 */
function formatPeak(memory: RenderMemoryReport): string {
  return isReported(memory.peakAllocated) ? formatBytes(memory.peakAllocated) : "Not reported";
}

/**
 * @param memory - What the renderer holds on the GPU.
 * @returns What the process holds in the GPU's own memory, DLSS's and the driver's counted, of what the system lets it
 *   hold now; or that the driver says none.
 */
function formatVideoMemory(memory: RenderMemoryReport): string {
  if (!isReported(memory.used)) {
    return "Not reported";
  }

  return isReported(memory.budget)
    ? `${formatBytes(memory.used)} used of ${formatBytes(memory.budget)} budget`
    : `${formatBytes(memory.used)} used`;
}

interface ILevelRendererResidentSectionProps extends BaseComponentProps {
  /** How far the renderer has read the level, or null until it says. */
  load: Nullable<RenderLoadReport>;
  /** What it holds on the GPU. */
  memory: RenderMemoryReport;
}

/**
 * What the level holds now, its sectors and textures, and what the GPU holds: the allocator's for everything the
 * renderer made and its peak, and the GPU's own memory as its driver tells it, all of it and what this process uses.
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
      <EditorPanelProperty label={"GPU memory"} value={formatAllocated(memory)} />
      <EditorPanelProperty label={"GPU memory peak"} value={formatPeak(memory)} />
      <EditorPanelProperty label={"Video memory"} value={formatVideoMemory(memory)} />
      <EditorPanelProperty
        label={"Video memory total"}
        value={isReported(memory.total) ? formatBytes(memory.total) : "Not reported"}
      />
    </EditorPanelSection>
  );
}
