import { Nullable } from "@xrf/types";

import { describeProcessCount, IMemoryFigures, sumWebviewMemory, toMemoryFigures } from "@/core/diagnostics/lib";
import { MemoryUsage, RuntimeSnapshot } from "@/core/ipc/types/xrf-app";
import { formatBytes } from "@/lib/memory/format";

/** One memory figure of the Runtime section, and what it is of. */
export interface IRuntimeMemoryFigure {
  value: string;
  hint: string;
}

/**
 * Describes the backend's memory: in use with its commit, as the status bar counts it, or resident where the host
 * cannot say.
 *
 * @param snapshot - The runtime snapshot, whose resident figure is the fallback.
 * @param memory - The memory reading, or `null` where the host gave none.
 * @returns The figure.
 */
export function describeBackendMemory(snapshot: RuntimeSnapshot, memory: Nullable<MemoryUsage>): IRuntimeMemoryFigure {
  if (!memory) {
    return { value: formatBytes(snapshot.process.residentMemory), hint: "resident" };
  }

  const figures: IMemoryFigures = toMemoryFigures(memory.application);

  return { value: formatBytes(figures.inUse), hint: `${formatBytes(figures.committed)} committed` };
}

/**
 * Describes the webview's memory over all its processes, the same way as {@link describeBackendMemory}.
 *
 * @param snapshot - The runtime snapshot, whose descendant figure is the fallback.
 * @param memory - The memory reading, or `null` where the host gave none.
 * @returns The figure.
 */
export function describeWebviewMemory(snapshot: RuntimeSnapshot, memory: Nullable<MemoryUsage>): IRuntimeMemoryFigure {
  if (!memory?.webview.length) {
    return {
      value: formatBytes(snapshot.descendants.residentMemory),
      hint: `${describeProcessCount(snapshot.descendants.processes)}, resident`,
    };
  }

  const figures: IMemoryFigures = sumWebviewMemory(memory.webview);

  return {
    value: formatBytes(figures.inUse),
    hint: `${describeProcessCount(memory.webview.length)}, ${formatBytes(figures.committed)} committed`,
  };
}
