import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent, waitFor } from "@testing-library/react";

import { GamedataVerifierApplication } from "@/applications/gamedata-verifier/GamedataVerifierApplication";
import { GamedataVerifierService } from "@/applications/gamedata-verifier/services/verifier";
import { GamedataVerifySummary } from "@/core/ipc/types/xrf-app";
import { JobsService } from "@/core/jobs/services/jobs";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { noop } from "@/lib/callbacks/noop";

describe("GamedataVerifierApplication", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem("xrf.form.gamedata-verifier.gamedata", "C:\\gamedata");
    setMockInvokeResponses({});
  });

  it("starts on Verify and keeps cancellation visible until the backend stops", async () => {
    let resolve: (summary: GamedataVerifySummary) => void = noop;
    const response = new Promise<GamedataVerifySummary>((settle) => {
      resolve = settle;
    });

    setMockInvokeResponses({ "plugin:gamedata|verify_project": () => response });

    const container = mockContainer([GamedataVerifierService]);
    const view = renderWithProviders(<GamedataVerifierApplication />, { route: "/gamedata-verifier", container });
    const submit = await view.findByRole("button", { name: "Verify" });

    await waitFor(() => expect(submit).toBeEnabled());

    expect(view.getByRole("checkbox", { name: "Strict" })).not.toBeChecked();

    fireEvent.click(submit);

    const jobs = container.get(JobsService);
    const id = jobs.jobs[0]?.id;

    expect(mockInvoke).toHaveBeenCalledWith("plugin:gamedata|verify_project", {
      request: { root: "C:\\gamedata", checks: null, isStrict: false },
      jobId: id,
      progress: expect.anything(),
    });
    expect(view.getByRole("progressbar")).toBeInTheDocument();
    expect(submit).toBeDisabled();
    expect(view.getByRole("checkbox", { name: "Strict" })).toBeDisabled();

    fireEvent.click(view.getByRole("button", { name: "Cancel" }));

    expect(mockInvoke).toHaveBeenCalledWith("plugin:jobs|cancel", { id });
    expect(view.getByRole("button", { name: "Stopping" })).toBeDisabled();
    expect(jobs.jobs).toHaveLength(1);

    await act(async () => resolve({ outcome: "cancelled", status: "skipped", checks: [], duration: 0 }));

    expect(jobs.jobs).toHaveLength(0);
    expect(view.queryByRole("button", { name: "Stopping" })).not.toBeInTheDocument();
    expect(submit).toBeEnabled();

    fireEvent.click(view.getByRole("button", { name: "Show parameters" }));

    expect(view.getByRole("checkbox", { name: "Strict" })).toBeEnabled();
  });
});
