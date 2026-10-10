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
 * @param memory - What the renderer holds on the GPU.
 * @returns The most the GPU allocator held allocated since the renderer started, or that it reports none.
 */
function formatPeak(memory: RenderMemoryReport): string {
  const peak: unknown = memory.peakAllocated;

  return typeof peak === "number" && Number.isFinite(peak) ? formatBytes(peak) : "Not reported";
}

/**
 * @param bytes - A count of bytes the report may leave out.
 * @returns Whether it holds one.
 */
function isReported(bytes: unknown): bytes is number {
  return typeof bytes === "number" && Number.isFinite(bytes);
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
