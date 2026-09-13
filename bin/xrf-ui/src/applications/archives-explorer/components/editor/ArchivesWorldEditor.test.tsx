import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ArchivesExplorerApplication } from "@/applications/archives-explorer/ArchivesExplorerApplication";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import { ArchiveSubject } from "@/core/bindings/types/xrf-app";
import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { mockArchivedContainer, mockArchivesWorldSubject, mockArchiveWorldEntry } from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const OVERRIDDEN = mockArchiveWorldEntry({
  name: "configs\\system.ltx",
  shadowed: [mockArchivedContainer("C:\\game\\db\\configs.db0")],
});

const ARCHIVED = mockArchiveWorldEntry({
  container: mockArchivedContainer("C:\\game\\db\\scripts.db0"),
  name: "scripts\\actor.script",
  sizeReal: 1024,
});

const WORLD: ArchiveSubject = mockArchivesWorldSubject([OVERRIDDEN, ARCHIVED]);

describe("opened archives world", () => {
  beforeEach(() => {
    window.localStorage.clear();

    setMockInvokeResponses({
      ["plugin:archives|get_subject"]: mockSessionResponse(WORLD),
      ["plugin:archives|read_file"]: {
        name: OVERRIDDEN.name,
        content: "[system]",
        size: OVERRIDDEN.sizeReal,
      },
    });
  });

  function renderEditor() {
    return renderWithProviders(
      <>
        <ArchivesExplorerApplication />
        <ApplicationStatusBar />
      </>,
      { route: "/archives-explorer", bindings: [AssetService, ArchivesService] }
    );
  }

  it("counts its sources and the paths more than one of them answers", async () => {
    const { findByText } = renderEditor();

    expect(await findByText("2 sources")).toBeInTheDocument();
    expect(await findByText("1 overridden")).toBeInTheDocument();
  });

  it("says once that some files exist in more than one source", async () => {
    const { findByText } = renderEditor();

    expect(await findByText(/1 file\(s\) here exist in more than one source/, { exact: false })).toBeInTheDocument();
  });

  it("shows where a selected file is read from and what it hides", async () => {
    const { findByLabelText, findByText } = await act(async () =>
      renderWithProviders(
        <ApplicationShellFrame>
          <ArchivesExplorerApplication />
        </ApplicationShellFrame>,
        { route: "/archives-explorer", bindings: [AssetService, ArchivesService], hasShell: true }
      )
    );

    await userEvent.dblClick(await findByText("configs"));
    await userEvent.dblClick(await findByText("system.ltx"));

    await userEvent.click(await findByLabelText("File details"));

    // The fact the explorer could not state before this mode: the engine opens the loose copy, and the archived one
    // behind it is exactly where a person would look for the file and not find what the game reads.
    expect(await findByText("Loose file")).toBeInTheDocument();
    expect(await findByText("C:\\game\\gamedata\\configs\\system.ltx")).toBeInTheDocument();
    expect(await findByText("C:\\game\\db\\configs.db0")).toBeInTheDocument();
  });

  it("has no name-table details to show for a world", async () => {
    const { findByLabelText, findByText, queryByText } = await act(async () =>
      renderWithProviders(
        <ApplicationShellFrame>
          <ArchivesExplorerApplication />
        </ApplicationShellFrame>,
        { route: "/archives-explorer", bindings: [AssetService, ArchivesService], hasShell: true }
      )
    );

    await userEvent.dblClick(await findByText("scripts"));
    await userEvent.dblClick(await findByText("actor.script"));

    await userEvent.click(await findByLabelText("File details"));

    expect(await findByText("Archive")).toBeInTheDocument();
    // A loose file has no offset and a world has no shared payloads, so neither is shown rather than shown as zero.
    expect(queryByText("Offset")).not.toBeInTheDocument();
    expect(queryByText("Shared payload")).not.toBeInTheDocument();
  });

  it("marks a row the engine reads out of a volume, and leaves the loose copy plain", async () => {
    const { findByText, queryByText } = await act(async () =>
      renderWithProviders(
        <ApplicationShellFrame>
          <ArchivesExplorerApplication />
        </ApplicationShellFrame>,
        { route: "/archives-explorer", bindings: [AssetService, ArchivesService], hasShell: true }
      )
    );

    await userEvent.dblClick(await findByText("configs"));
    await userEvent.dblClick(await findByText("scripts"));

    expect(await findByText("actor.script")).toBeInTheDocument();
    expect(await findByText("db")).toBeInTheDocument();

    expect(await findByText("system.ltx")).toBeInTheDocument();
    expect(queryByText("Loose file")).not.toBeInTheDocument();
  });
});
