import { ABSENT_VALUE, formatBytes, formatBytesCompact } from "@xrf/format";
import { Nullable } from "@xrf/types";

import { IMemoryFigures, sumWebviewMemory, toMemoryFigures } from "@/core/diagnostics/lib/memory-figures";
import {
  EWebviewProcessKind,
  GpuMemoryUsage,
  MemoryUsage,
  WebviewProcessKind,
  WebviewProcessMemory,
} from "@/core/ipc/types/xrf-app-system";
import { IEditorStatusDetail } from "@/core/shell/editor-shell/editor-status-detail";
import { IEditorStatusSegment } from "@/core/shell/editor-shell/editor-status-segment";

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
 * Describes a memory reading for the status bar: what the backend, the webview and video memory have in use, `B`, `W`
 * and `G` for short, each left out of the text where it was not read; and in its hover each webview process kind under
 * the webview, what the renderer allocated and the budget under video memory, the commit of both processes, then the
 * caller's rows.
 *
 * @param usage - One reading of the backend, the webview's processes and the GPU.
 * @param details - Further hover rows, in order, after the commit.
 * @returns The segment to publish.
 */
export function describeMemoryUsage(
  usage: MemoryUsage,
  details: ReadonlyArray<IEditorStatusDetail>
): IEditorStatusSegment {
  const backend: IMemoryFigures = toMemoryFigures(usage.application);
  const webview: Nullable<IMemoryFigures> = usage.webview.length ? sumWebviewMemory(usage.webview) : null;
  const videoMemory: Nullable<number> = usage.gpu?.used ?? null;

  return {
    id: MEMORY_USAGE_SEGMENT_ID,
    text: [
      `B ${formatBytesCompact(backend.inUse)}`,
      webview ? `W ${formatBytesCompact(webview.inUse)}` : null,
      videoMemory === null ? null : `G ${formatBytesCompact(videoMemory)}`,
    ]
      .filter((figure: Nullable<string>): figure is string => figure !== null)
      .join(" "),
    details: [
      { label: "Backend", value: formatBytes(backend.inUse) },
      { label: "Webview", value: webview ? formatBytes(webview.inUse) : ABSENT_VALUE },
      ...groupWebviewProcesses(usage.webview).map((group: IWebviewProcessGroup): IEditorStatusDetail => ({
        label: describeWebviewProcessGroup(group),
        value: formatBytes(group.memory.inUse),
        isNested: true,
      })),
      ...describeVideoMemory(usage.gpu),
      { label: "Committed", value: formatBytes(backend.committed + (webview?.committed ?? 0)) },
      ...details,
    ],
  };
}

/** What the process holds in video memory, then what the renderer allocated there and the budget, each where read. */
function describeVideoMemory(gpu: Nullable<GpuMemoryUsage>): Array<IEditorStatusDetail> {
  if (!gpu) {
    return [{ label: "Video memory", value: ABSENT_VALUE }];
  }

  const rows: Array<IEditorStatusDetail> = [
    { label: "Video memory", value: gpu.used === null ? ABSENT_VALUE : formatBytes(gpu.used) },
  ];

  if (gpu.allocated !== null) {
    rows.push({ label: "Allocated", value: formatBytes(gpu.allocated), isNested: true });
  }

  if (gpu.budget !== null) {
    rows.push({ label: "Budget", value: formatBytes(gpu.budget), isNested: true });
  }

  return rows;
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
