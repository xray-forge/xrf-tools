import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ArchivesExplorerApplication } from "@/applications/archives-explorer/ArchivesExplorerApplication";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import { mockArchiveResolution, mockArchivesWorldSubject } from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

async function renderExplorer() {
  return act(async () =>
    renderWithProviders(
      <ApplicationShellFrame>
        <ArchivesExplorerApplication />
      </ApplicationShellFrame>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService], hasShell: true }
    )
  );
}

describe("archive resolution dialog", () => {
  beforeEach(() => {
    window.localStorage.clear();

    setMockInvokeResponses({
      ["plugin:archives|describe_resolution"]: mockArchiveResolution(),
      ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesWorldSubject()),
    });
  });

  it("stays closed until the toolbar action is used", async () => {
    const { findByLabelText, queryByRole } = await renderExplorer();

    expect(queryByRole("dialog", { name: "Resolution" })).toBeNull();

    await userEvent.click(await findByLabelText("Resolution"));

    expect(await findByLabelText("Close resolution")).toBeInTheDocument();
  });

  it("lists the sources in the order they are searched", async () => {
    const { findByLabelText, getAllByTestId } = await renderExplorer();

    await userEvent.click(await findByLabelText("Resolution"));
    await findByLabelText("Close resolution");

    // The whole point of the dialog: the loose tree the engine prefers stands above the volumes it overrides, and the
    // row says which declaration put it there rather than only where it is.
    expect(getAllByTestId("archive-resolution-source-row").map((row: HTMLElement) => row.textContent)).toEqual([
      expect.stringContaining("$game_data$"),
      expect.stringContaining("$arch_dir$"),
    ]);

    expect(getAllByTestId("archive-resolution-source-row")[0]?.textContent).toContain("Files");
    expect(getAllByTestId("archive-resolution-source-row")[1]?.textContent).toContain("Archives");
  });

  it("lists the volumes of a set in the order a lookup reaches them", async () => {
    const { findByLabelText, getAllByTestId } = await renderExplorer();

    await userEvent.click(await findByLabelText("Resolution"));
    await findByLabelText("Close resolution");

    // A set merges with the later volume winning, so the patch volume is asked before the one it patches. Listing them
    // in merge order would say the opposite of what the reads do.
    expect(getAllByTestId("archive-resolution-volume-row").map((row: HTMLElement) => row.textContent)).toEqual([
      expect.stringContaining("patch.db1"),
      expect.stringContaining("textures.db0"),
    ]);
  });

  it("names a declared source that could not be opened", async () => {
    const { findByLabelText, findByTestId, findByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Resolution"));

    // Absent content reads exactly like content that was never there, so an unopened source is stated rather than
    // silently missing from the order above it.
    expect(await findByTestId("archive-resolution-unread-section")).toBeInTheDocument();
    expect(await findByText("$arch_dir_levels$")).toBeInTheDocument();
    expect(await findByText("failed to read archive header")).toBeInTheDocument();
  });

  it("asks once and keeps the answer across openings", async () => {
    let calls: number = 0;

    setMockInvokeResponses({
      ["plugin:archives|describe_resolution"]: () => {
        calls += 1;

        return mockArchiveResolution();
      },
      ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesWorldSubject()),
    });

    const { findByLabelText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Resolution"));
    await findByLabelText("Close resolution");

    await userEvent.click(await findByLabelText("Close resolution"));
    await userEvent.click(await findByLabelText("Resolution"));
    await findByLabelText("Close resolution");

    // The service keeps what it loaded for as long as the subject is open, so reopening costs nothing.
    expect(calls).toBe(1);
  });
});
