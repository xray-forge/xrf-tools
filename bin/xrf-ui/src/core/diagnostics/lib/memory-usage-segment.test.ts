import { describe, expect, it } from "@jest/globals";

import { describeMemoryUsage } from "@/core/diagnostics/lib/memory-usage-segment";
import { EWebviewProcessKind, MemoryUsage, WebviewProcessKind, WebviewProcessMemory } from "@/core/ipc/types/xrf-app";
import { IEditorStatusSegment } from "@/core/shell/editor-shell/editor-status-segment";

const MB: number = 1024 * 1024;

function process(kind: WebviewProcessKind, inUse: number, committed: number, pid: number = 1): WebviewProcessMemory {
  return {
    kind,
    pid,
    memory: { committed: committed * MB, workingSet: (inUse + 10) * MB, privateWorkingSet: inUse * MB },
  };
}

function usage(webview: Array<WebviewProcessMemory>): MemoryUsage {
  return {
    application: { committed: 410 * MB, workingSet: 300 * MB, privateWorkingSet: 260 * MB },
    webview,
  };
}

describe("describeMemoryUsage", () => {
  it("says what the backend and the whole webview have in use, not what they committed", () => {
    const segment: IEditorStatusSegment = describeMemoryUsage(
      usage([
        process(EWebviewProcessKind.GPU, 766, 3667),
        process(EWebviewProcessKind.RENDERER, 2000, 2400),
        process(EWebviewProcessKind.BROWSER, 106, 150),
      ]),
      []
    );

    expect(segment.text).toBe("B 260MB W 2.8GB");
  });

  it("lists the webview's kinds under it, most in use first, then the commit of both", () => {
    const segment: IEditorStatusSegment = describeMemoryUsage(
      usage([
        process(EWebviewProcessKind.BROWSER, 90, 120, 1),
        process(EWebviewProcessKind.UTILITY, 20, 30, 2),
        process(EWebviewProcessKind.GPU, 480, 3686, 3),
        process(EWebviewProcessKind.UTILITY, 20, 30, 4),
        process(EWebviewProcessKind.RENDERER, 2600, 2900, 5),
      ]),
      []
    );

    expect(segment.details).toEqual([
      { label: "Backend", value: "260 MB" },
      { label: "Webview", value: "3.13 GB" },
      { label: "Renderer", value: "2.54 GB", isNested: true },
      { label: "GPU", value: "480 MB", isNested: true },
      { label: "Browser", value: "90 MB", isNested: true },
      { label: "Utility ×2", value: "40 MB", isNested: true },
      { label: "Committed", value: "7.01 GB" },
    ]);
  });

  it("counts the whole working set where the host reports no private one", () => {
    const segment: IEditorStatusSegment = describeMemoryUsage(
      {
        application: { committed: 410 * MB, workingSet: 300 * MB, privateWorkingSet: null },
        webview: [
          {
            kind: EWebviewProcessKind.GPU,
            pid: 1,
            memory: { committed: 900 * MB, workingSet: 500 * MB, privateWorkingSet: null },
          },
        ],
      },
      []
    );

    expect(segment.text).toBe("B 300MB W 500MB");
  });

  it("adds the caller's rows after the commit, in order", () => {
    const segment: IEditorStatusSegment = describeMemoryUsage(usage([process(EWebviewProcessKind.GPU, 1, 1)]), [
      { label: "Script heap", value: "42 MB / 4 GB" },
      { label: "Renderer copies", value: "8 MB" },
    ]);

    expect(segment.details.slice(-3)).toEqual([
      { label: "Committed", value: "411 MB" },
      { label: "Script heap", value: "42 MB / 4 GB" },
      { label: "Renderer copies", value: "8 MB" },
    ]);
  });

  it("says only the backend when the webview reported no processes, and still adds the caller's rows", () => {
    const segment: IEditorStatusSegment = describeMemoryUsage(usage([]), [
      { label: "Script heap", value: "42 MB / 4 GB" },
    ]);

    expect(segment.text).toBe("B 260MB");
    expect(segment.details).toEqual([
      { label: "Backend", value: "260 MB" },
      { label: "Webview", value: "—" },
      { label: "Committed", value: "410 MB" },
      { label: "Script heap", value: "42 MB / 4 GB" },
    ]);
  });

  it("names every reading the same segment, whatever it says", () => {
    const first: IEditorStatusSegment = describeMemoryUsage(usage([]), []);
    const second: IEditorStatusSegment = describeMemoryUsage(usage([process(EWebviewProcessKind.GPU, 480, 900)]), []);

    expect(second.text).not.toBe(first.text);
    expect(second.id).toBe(first.id);
  });
});
