import { afterEach, describe, expect, it } from "@jest/globals";
import { renderHook, waitFor } from "@testing-library/react";

import { TMemoryDetailSource } from "@/core/diagnostics/lib/memory-detail-source";
import { useMemoryUsageSegment } from "@/core/diagnostics/lib/use-memory-usage-segment";
import { MemoryUsage } from "@/core/ipc/types/xrf-app";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";

const MB: number = 1024 * 1024;

const USAGE: MemoryUsage = {
  application: { committed: 410 * MB, workingSet: 300 * MB, privateWorkingSet: 260 * MB },
  webview: [],
};

describe("useMemoryUsageSegment", () => {
  afterEach(() => {
    resetMockInvoke();
  });

  it("adds each source's row to the hover, in order, and leaves out a source with nothing to say", async () => {
    setMockInvokeResponses({ "plugin:system|get_memory_usage": USAGE });

    const sources: ReadonlyArray<TMemoryDetailSource> = [
      () => ({ label: "First", value: "1 MB" }),
      () => null,
      () => ({ label: "Second", value: "2 MB" }),
    ];
    const { result } = renderHook(() => useMemoryUsageSegment(sources));

    await waitFor(() => expect(result.current?.text).toBe("B 260MB"));
    expect(result.current?.details.slice(-2)).toEqual([
      { label: "First", value: "1 MB" },
      { label: "Second", value: "2 MB" },
    ]);
  });

  it("publishes nothing on a platform that cannot read memory", async () => {
    let reads: number = 0;

    setMockInvokeResponses({
      "plugin:system|get_memory_usage": () => {
        reads += 1;

        return null;
      },
    });

    const { result } = renderHook(() => useMemoryUsageSegment([() => ({ label: "Unused", value: "0 B" })]));

    await waitFor(() => expect(reads).toBe(1));
    expect(result.current).toBeNull();
  });
});
