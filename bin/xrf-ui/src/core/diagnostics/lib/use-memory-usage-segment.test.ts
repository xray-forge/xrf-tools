import { afterEach, describe, expect, it } from "@jest/globals";
import { renderHook, waitFor } from "@testing-library/react";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { MemoryUsage } from "@xrf/ipc/types/xrf-app-system";

import { TMemoryDetailSource } from "@/core/diagnostics/lib/memory-detail-source";
import { useMemoryUsageSegment } from "@/core/diagnostics/lib/use-memory-usage-segment";

const MB: number = 1024 * 1024;

const USAGE: MemoryUsage = {
  application: { committed: 410 * MB, workingSet: 300 * MB, privateWorkingSet: 260 * MB },
  webview: [],
  gpu: null,
};

describe("useMemoryUsageSegment", () => {
  afterEach(() => {
    resetMockInvoke();
  });

  it("adds each source's row to the hover, in order, and leaves out a source with nothing to say", async () => {
    mockIpcCommands({ [ESystemCommand.GET_MEMORY_USAGE]: USAGE });

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

    mockIpcCommands({
      [ESystemCommand.GET_MEMORY_USAGE]: () => {
        reads += 1;

        return null;
      },
    });

    const { result } = renderHook(() => useMemoryUsageSegment([() => ({ label: "Unused", value: "0 B" })]));

    await waitFor(() => expect(reads).toBe(1));
    expect(result.current).toBeNull();
  });
});
