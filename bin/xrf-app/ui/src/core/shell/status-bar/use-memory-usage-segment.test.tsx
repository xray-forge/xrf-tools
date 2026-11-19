import { afterEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { MemoryUsage } from "@xrf/ipc/types/xrf-app-system";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { TMemoryDetailSource } from "@/core/shell/status-bar/memory-detail-source";
import { IShellStatusDetail } from "@/core/shell/status-bar/shell-status-detail";
import { IShellStatusSegment } from "@/core/shell/status-bar/shell-status-segment";
import { useMemoryUsageSegment } from "@/core/shell/status-bar/use-memory-usage-segment";
import { renderWithProviders } from "@/fixtures/utils/render";

const MB: number = 1024 * 1024;

const USAGE: MemoryUsage = {
  application: { committed: 410 * MB, workingSet: 300 * MB, privateWorkingSet: 260 * MB },
  webview: [],
  gpu: null,
};

const SOURCES: ReadonlyArray<TMemoryDetailSource> = [
  () => ({ label: "First", value: "1 MB" }),
  () => null,
  () => ({ label: "Second", value: "2 MB" }),
];

function Segment(): ReactElement {
  const segment: Nullable<IShellStatusSegment> = useMemoryUsageSegment(SOURCES);

  return segment ? (
    <>
      <span data-testid={"text"}>{segment.text}</span>
      <span data-testid={"details"}>
        {segment.details.map((detail: IShellStatusDetail) => `${detail.label}=${detail.value}`).join(";")}
      </span>
    </>
  ) : (
    <span data-testid={"empty"} />
  );
}

describe("useMemoryUsageSegment", () => {
  afterEach(() => {
    resetMockInvoke();
  });

  it("adds each source's row to the hover, in order, and leaves out a source with nothing to say", async () => {
    mockIpcCommands({ [ESystemCommand.GET_MEMORY_USAGE]: USAGE });

    const { findByTestId, getByTestId } = renderWithProviders(<Segment />);

    expect((await findByTestId("text")).textContent).toBe("B 260MB");
    expect(getByTestId("details").textContent).toMatch(/First=1 MB;Second=2 MB$/);
  });

  it("publishes nothing on a platform that cannot read memory", async () => {
    let reads: number = 0;

    mockIpcCommands({
      [ESystemCommand.GET_MEMORY_USAGE]: () => {
        reads += 1;

        return null;
      },
    });

    const { getByTestId } = renderWithProviders(<Segment />);

    await waitFor(() => expect(reads).toBe(1));
    expect(getByTestId("empty")).toBeInTheDocument();
  });
});
