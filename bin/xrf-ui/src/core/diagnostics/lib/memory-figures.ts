import { ProcessMemory, WebviewProcessMemory } from "@/core/ipc/types/xrf-app";

/** What a process, or several summed, holds: what it occupies alone, and what is charged to it. */
export interface IMemoryFigures {
  /** Private working set, which Task Manager's Memory column shows; the whole working set where the host lacks it. */
  inUse: number;
  /** Private commit, resident or not: GPU driver backing lands here without being in use. */
  committed: number;
}

/**
 * @param memory - One process's reading.
 * @returns Its figures.
 */
export function toMemoryFigures(memory: ProcessMemory): IMemoryFigures {
  return { inUse: memory.privateWorkingSet ?? memory.workingSet, committed: memory.committed };
}

/**
 * @param processes - Webview processes to sum.
 * @returns Their figures, summed.
 */
export function sumWebviewMemory(processes: ReadonlyArray<WebviewProcessMemory>): IMemoryFigures {
  return processes.reduce(
    (sum: IMemoryFigures, process: WebviewProcessMemory) => addMemoryFigures(sum, toMemoryFigures(process.memory)),
    { inUse: 0, committed: 0 }
  );
}

function addMemoryFigures(first: IMemoryFigures, second: IMemoryFigures): IMemoryFigures {
  return { inUse: first.inUse + second.inUse, committed: first.committed + second.committed };
}
