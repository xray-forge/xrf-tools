import { Nullable } from "@xrf/types";

import { IEditorStatusDetail } from "@/core/shell/editor-shell/editor-status-detail";
import { formatBytes } from "@/lib/memory/format";

/**
 * Memory hover row for the CPU copies the level's renderer keeps of what it put on the GPU.
 *
 * @param memory - Bytes the renderer last said it holds.
 * @returns The row, or `null` before the renderer said it holds anything.
 */
export function toLevelRendererMemoryDetail(memory: number): Nullable<IEditorStatusDetail> {
  return memory > 0 ? { label: "Renderer copies", value: formatBytes(memory) } : null;
}
