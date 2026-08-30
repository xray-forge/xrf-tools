import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { JobDescription } from "@/core/bindings/types/xrf-app";
import { JobsPanel } from "@/core/shell/panel/jobs/JobsPanel";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const LIST_COMMAND: string = "plugin:jobs|list";
const CANCEL_COMMAND: string = "plugin:jobs|cancel";

function described(patch: Partial<JobDescription> = {}): JobDescription {
  return {
    id: "b8f0",
    kind: "archives.pack",
    leaseKeys: ["archives.pack:c:\\out|gamedata"],
    isCancelRequested: false,
    request: null,
    progress: {
      levels: [{ id: "write", label: null, completed: 7, total: 10, unit: "items" }],
      duration: 2000,
      detail: null,
    },
    conclusion: null,
    error: null,
    result: null,
    duration: 2000,
    ...patch,
  } as JobDescription;
}

describe("JobsPanel", () => {
  beforeEach(() => {
    resetMockInvoke();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("names the work rather than the kind that addressed it", async () => {
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const rendered: RenderResult = renderWithProviders(<JobsPanel />);

    expect(await rendered.findByText("Archive packing")).toBeInTheDocument();
  });

  it("shows the leases a running job holds, which is why another was refused", async () => {
    // The question this panel exists to answer: a refused start names a job the user cannot otherwise see.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const rendered: RenderResult = renderWithProviders(<JobsPanel />);

    expect(await rendered.findByText("archives.pack:c:\\out|gamedata")).toBeInTheDocument();
  });

  it("reports how a finished job ended", async () => {
    setMockInvokeResponses({ [LIST_COMMAND]: [described({ conclusion: "failed", error: "volume cap refused" })] });

    const rendered: RenderResult = renderWithProviders(<JobsPanel />);

    expect(await rendered.findByText("failed")).toBeInTheDocument();
    expect(await rendered.findByText("volume cap refused")).toBeInTheDocument();
  });

  it("says plainly when there is nothing to show", async () => {
    setMockInvokeResponses({ [LIST_COMMAND]: [] });

    const rendered: RenderResult = renderWithProviders(<JobsPanel />);

    expect(await rendered.findByText(/Nothing is running/)).toBeInTheDocument();
  });

  it("offers no cancel for a job that has already ended", async () => {
    setMockInvokeResponses({
      [LIST_COMMAND]: [described({ conclusion: "completed" })],
      [CANCEL_COMMAND]: true,
    });

    const rendered: RenderResult = renderWithProviders(<JobsPanel />);

    await rendered.findByText("completed");

    expect(rendered.queryByRole("button", { name: "Cancel" })).toBeNull();
  });

  it("keeps the last answer when the backend cannot be read", async () => {
    // The backend being briefly unreadable is not evidence that nothing is running, and a list that emptied itself
    // would say it was.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const rendered: RenderResult = renderWithProviders(<JobsPanel />);

    await rendered.findByText("Archive packing");

    setMockInvokeResponses({
      [LIST_COMMAND]: () => {
        throw new Error("state unavailable");
      },
    });

    await jest.advanceTimersByTimeAsync(1000);

    expect(rendered.queryByText("Archive packing")).toBeInTheDocument();
  });
});
