import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ArchivesExplorerApplication } from "@/applications/archives-explorer/ArchivesExplorerApplication";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import { ArchiveSubject } from "@/core/ipc/types/xrf-app";
import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import {
  mockArchiveStatistics,
  mockArchivesVolumes,
  mockArchivesWorldSubject,
  mockArchiveWorldStatistics,
} from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

/** Awaited, so the restore flow has settled and the editor's own toolbar exists before anything looks for it. */
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

function openVolumes(subject: ArchiveSubject = mockArchivesVolumes()): void {
  setMockInvokeResponses({
    ["plugin:archives|describe_statistics"]: mockArchiveStatistics(),
    ["plugin:archives|get_subject"]: mockSessionResponse(subject),
  });
}

describe("archive statistics dialog", () => {
  beforeEach(() => {
    window.localStorage.clear();
    openVolumes();
  });

  it("stays closed until the toolbar action is used", async () => {
    const { findByLabelText, queryByRole } = await renderExplorer();

    // The command is not sent for a session that never asks: the answer costs a walk of the whole listing.
    expect(queryByRole("dialog", { name: "Statistics" })).toBeNull();

    await userEvent.click(await findByLabelText("Statistics"));

    expect(await findByLabelText("Close statistics")).toBeInTheDocument();
  });

  it("gives every section its own identity on the rendered surface", async () => {
    const { findByLabelText, findByTestId, findByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Statistics"));

    // The ids are destructure defaults rather than literals, so a wrong one renders `undefined` and nothing else
    // notices. Two sections are enough to prove the default reaches the surface it names.
    expect(await findByTestId("archive-overview-section")).toBeInTheDocument();

    await userEvent.click(await findByText("Extensions"));

    expect(await findByTestId("archive-extensions-section")).toBeInTheDocument();
  });

  it("reports both measurements for every extension", async () => {
    const { findByLabelText, findByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Statistics"));
    await userEvent.click(await findByText("Extensions"));

    // Both columns are always present: the one the view is not ordered by is needed as often as the one it is.
    expect(await findByText("dds")).toBeInTheDocument();
    expect(await findByText("4 KB")).toBeInTheDocument();
    expect(await findByText("4")).toBeInTheDocument();
  });

  it("reorders the rows when the measurement changes", async () => {
    const { findByLabelText, findByText, getAllByTestId } = await renderExplorer();

    await userEvent.click(await findByLabelText("Statistics"));
    await userEvent.click(await findByText("Extensions"));

    // The fixture ranks the two inversely on purpose: one large dds against four small ltx. A toggle that moved the
    // bars but left the order alone would be worse than no toggle, because the list would contradict what it draws.
    expect(getAllByTestId("stat-breakdown-row").map((row: HTMLElement) => row.textContent)).toEqual([
      expect.stringContaining("dds"),
      expect.stringContaining("ltx"),
      expect.stringContaining("som"),
    ]);

    await userEvent.click(await findByText("Count"));

    expect(getAllByTestId("stat-breakdown-row").map((row: HTMLElement) => row.textContent)).toEqual([
      expect.stringContaining("ltx"),
      expect.stringContaining("som"),
      expect.stringContaining("dds"),
    ]);
  });

  it("orders size bands by the measurement as well", async () => {
    const { findByLabelText, findByText, getAllByTestId } = await renderExplorer();

    await userEvent.click(await findByLabelText("Statistics"));
    await userEvent.click(await findByText("Sizes"));

    // The fixture's two bands rank inversely: the smaller band holds six files, the larger holds one big one. Each row
    // names the band it covers, so nothing is lost by ordering them like every other breakdown.
    expect(getAllByTestId("stat-breakdown-row").map((row: HTMLElement) => row.textContent)).toEqual([
      expect.stringContaining("1 KB - 4 KB"),
      expect.stringContaining("1 B - 1 KB"),
    ]);

    await userEvent.click(await findByText("Count"));

    expect(getAllByTestId("stat-breakdown-row").map((row: HTMLElement) => row.textContent)).toEqual([
      expect.stringContaining("1 B - 1 KB"),
      expect.stringContaining("1 KB - 4 KB"),
    ]);
  });

  it("marks a spelling the tooling does not declare", async () => {
    const { findByLabelText, findByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Statistics"));
    await userEvent.click(await findByText("Extensions"));

    // The row worth acting on. Folding it into an "other" bucket would hide a real X-Ray format the vocabulary misses.
    expect(await findByText("som")).toBeInTheDocument();
    expect(await findByText("unknown")).toBeInTheDocument();
  });

  it("offers a volume set the sections only it can answer", async () => {
    const { findByLabelText, findByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Statistics"));

    expect(await findByText("Compression")).toBeInTheDocument();
    expect(await findByText("Volumes")).toBeInTheDocument();

    // A volume set is an ordered stack of volumes exactly as a world is an ordered stack of mounts, so it answers
    // where its entries come from and what its own merge buried. It used to be offered neither, because the merge
    // discarded the evidence rather than because the question did not apply.
    expect(await findByText("Origins")).toBeInTheDocument();
  });

  it("offers a world the sections only it can answer", async () => {
    setMockInvokeResponses({
      ["plugin:archives|describe_statistics"]: mockArchiveWorldStatistics(),
      ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesWorldSubject()),
    });

    const { findByLabelText, findByText, queryByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Statistics"));

    expect(await findByText("Origins")).toBeInTheDocument();
    expect(await findByText("Overrides")).toBeInTheDocument();

    // A loose file has no stored size, so no compression figure over a mixed tree would be honest.
    expect(queryByText("Compression")).toBeNull();
    expect(queryByText("Volumes")).toBeNull();
  });

  it("says what an override arrangement costs, apart from what the engine loads", async () => {
    setMockInvokeResponses({
      ["plugin:archives|describe_statistics"]: mockArchiveWorldStatistics(),
      ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesWorldSubject()),
    });

    const { findByLabelText, findByText } = await renderExplorer();

    await userEvent.click(await findByLabelText("Statistics"));
    await userEvent.click(await findByText("Overrides"));

    expect(await findByText("8 KB hidden")).toBeInTheDocument();
  });
});
