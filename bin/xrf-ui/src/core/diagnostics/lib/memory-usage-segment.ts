import { IMemoryFigures, sumWebviewMemory, toMemoryFigures } from "@/core/diagnostics/lib/memory-figures";
import { EWebviewProcessKind, MemoryUsage, WebviewProcessKind, WebviewProcessMemory } from "@/core/ipc/types/xrf-app";
import { IEditorStatusDetail } from "@/core/shell/editor-shell/editor-status-detail";
import { IEditorStatusSegment } from "@/core/shell/editor-shell/editor-status-segment";
import { formatBytes } from "@/lib/memory/format";

/** Names the memory segment across readings, so its hover stays open while the figures change. */
const MEMORY_USAGE_SEGMENT_ID: string = "memory-usage";

/** What each webview process kind is called where a person reads it. */
const WEBVIEW_PROCESS_LABELS: Record<WebviewProcessKind, string> = {
  [EWebviewProcessKind.BROWSER]: "Browser",
  [EWebviewProcessKind.RENDERER]: "Renderer",
  [EWebviewProcessKind.GPU]: "GPU",
  [EWebviewProcessKind.UTILITY]: "Utility",
  [EWebviewProcessKind.SANDBOX_HELPER]: "Sandbox helper",
  [EWebviewProcessKind.PPAPI_PLUGIN]: "Plugin",
  [EWebviewProcessKind.PPAPI_BROKER]: "Plugin broker",
  [EWebviewProcessKind.OTHER]: "Other",
};

/** Every webview process of one kind, summed. */
interface IWebviewProcessGroup {
  kind: WebviewProcessKind;
  memory: IMemoryFigures;
  processes: number;
}

/**
 * Describes a memory reading for the status bar: what the backend and the webview have in use, each webview process
 * kind under it, the commit of both, then the caller's rows.
 *
 * @param usage - One reading of the backend and the webview's processes.
 * @param details - Further hover rows, in order, after the commit.
 * @returns The segment to publish.
 */
export function describeMemoryUsage(
  usage: MemoryUsage,
  details: ReadonlyArray<IEditorStatusDetail>
): IEditorStatusSegment {
  const backend: IMemoryFigures = toMemoryFigures(usage.application);
  const backendRow: IEditorStatusDetail = { label: "Backend", value: formatBytes(backend.inUse) };

  if (!usage.webview.length) {
    return {
      id: MEMORY_USAGE_SEGMENT_ID,
      text: `Backend ${formatBytes(backend.inUse)}`,
      details: [
        backendRow,
        { label: "Webview", value: "—" },
        { label: "Committed", value: formatBytes(backend.committed) },
        ...details,
      ],
    };
  }

  const webview: IMemoryFigures = sumWebviewMemory(usage.webview);

  return {
    id: MEMORY_USAGE_SEGMENT_ID,
    text: `Backend ${formatBytes(backend.inUse)} · Webview ${formatBytes(webview.inUse)}`,
    details: [
      backendRow,
      { label: "Webview", value: formatBytes(webview.inUse) },
      ...groupWebviewProcesses(usage.webview).map((group: IWebviewProcessGroup): IEditorStatusDetail => ({
        label: describeWebviewProcessGroup(group),
        value: formatBytes(group.memory.inUse),
        isNested: true,
      })),
      { label: "Committed", value: formatBytes(backend.committed + webview.committed) },
      ...details,
    ],
  };
}

/** Sums the processes of each kind, most in use first. */
function groupWebviewProcesses(processes: ReadonlyArray<WebviewProcessMemory>): Array<IWebviewProcessGroup> {
  const byKind: Map<WebviewProcessKind, Array<WebviewProcessMemory>> = new Map();

  for (const process of processes) {
    const group: Array<WebviewProcessMemory> | undefined = byKind.get(process.kind);

    if (group) {
      group.push(process);
    } else {
      byKind.set(process.kind, [process]);
    }
  }

  return [...byKind.entries()]
    .map(([kind, group]: [WebviewProcessKind, Array<WebviewProcessMemory>]): IWebviewProcessGroup => ({
      kind,
      memory: sumWebviewMemory(group),
      processes: group.length,
    }))
    .sort((first: IWebviewProcessGroup, second: IWebviewProcessGroup) => second.memory.inUse - first.memory.inUse);
}

/** The kind's name, with its process count where it has several, such as `Utility ×2`. */
function describeWebviewProcessGroup(group: IWebviewProcessGroup): string {
  const label: string = WEBVIEW_PROCESS_LABELS[group.kind];

  return group.processes > 1 ? `${label} ×${group.processes}` : label;
}
