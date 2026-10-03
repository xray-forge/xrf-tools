import { TMemoryDetailSource } from "@/core/diagnostics/lib";
import { RenderMemoryReport } from "@/core/ipc/types/xrf-renderer";
import { formatBytes } from "@/lib/memory/format";

/**
 * Memory hover rows for what the renderer holds on the GPU: textures, and the scene's buffers.
 *
 * @param read - Reads the memory the renderer last reported.
 * @returns A source for each row, each saying nothing while the renderer holds none of it.
 */
export function toNativeMemoryDetails(read: () => RenderMemoryReport): Array<TMemoryDetailSource> {
  return [
    () => {
      const bytes: number = read().textures;

      return bytes > 0 ? { label: "GPU textures", value: formatBytes(bytes) } : null;
    },
    () => {
      const bytes: number = read().scene;

      return bytes > 0 ? { label: "GPU scene buffers", value: formatBytes(bytes) } : null;
    },
  ];
}
