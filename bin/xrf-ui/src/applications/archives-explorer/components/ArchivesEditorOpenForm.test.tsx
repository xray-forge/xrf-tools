import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { open } from "@tauri-apps/plugin-dialog";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { ArchivesEditorOpenForm } from "@/applications/archives-explorer/components/ArchivesEditorOpenForm";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

const ARCHIVES_DIRECTORY: string = "C:\\game\\database";
const ARCHIVE_VOLUME: string = "C:\\downloads\\gamedata.db0";
const INSTALLATION: string = "C:\\game";

describe("ArchivesEditorOpenForm", () => {
  const mockOpen = jest.mocked(open<{ multiple: false }>);

  beforeEach(() => {
    window.localStorage.clear();

    setMockInvokeResponses({
      ["plugin:archives|get_subject"]: mockSessionResponse(null),
      ["plugin:archives|open_volumes"]: mockSessionResponse(null),
      ["plugin:archives|open_world"]: mockSessionResponse(null),
    });
  });

  function renderForm(): RenderResult {
    const container: Container = mockContainer([AssetService, ArchivesService]);

    return renderWithProviders(<ArchivesEditorOpenForm />, { route: "/archives-explorer", container });
  }

  it("asks for a directory in directory mode", async () => {
    mockOpen.mockResolvedValue(ARCHIVES_DIRECTORY);

    const { getByLabelText, getByText } = renderForm();

    expect(getByText("Indexes every archive in the directory for browsing.")).toBeInTheDocument();

    await userEvent.click(getByLabelText("Browse"));

    expect(open).toHaveBeenCalledWith({
      title: "Select archives directory",
      filters: undefined,
      directory: true,
      // Nothing is configured and nothing has been picked here yet, so the host decides where to open.
      defaultPath: undefined,
    });
  });

  it("associates the open mode with its visible label and description", async () => {
    const { findByRole } = renderForm();

    expect(await findByRole("group", { name: "Open" })).toHaveAccessibleDescription(
      "Browse a whole directory, one archive on its own, or the game as the engine mounts it"
    );
  });

  it("asks for a file, filtered to volumes, in archive mode", async () => {
    mockOpen.mockResolvedValue(ARCHIVE_VOLUME);

    const { getByLabelText, getByText } = renderForm();

    await userEvent.click(getByLabelText("Open archive"));

    expect(getByText("Indexes one archive volume for browsing.")).toBeInTheDocument();

    await userEvent.click(getByLabelText("Browse"));

    // Directory-only is the whole defect: a native dialog offering directories cannot select `gamedata.db0` at all.
    expect(open).toHaveBeenCalledWith({
      title: "Select archive volume",
      filters: [
        {
          name: "Archive volume",
          extensions: [
            "db",
            "db0",
            "db1",
            "db2",
            "db3",
            "db4",
            "db5",
            "db6",
            "db7",
            "db8",
            "db9",
            "xdb",
            "xdb0",
            "xdb1",
            "xdb2",
            "xdb3",
            "xdb4",
            "xdb5",
            "xdb6",
            "xdb7",
            "xdb8",
            "xdb9",
          ],
        },
        { name: "All files", extensions: ["*"] },
      ],
      directory: false,
    });
  });

  it("opens the volume that was picked, not its directory", async () => {
    mockOpen.mockResolvedValue(ARCHIVE_VOLUME);

    const { getByLabelText, getByRole } = renderForm();

    await userEvent.click(getByLabelText("Open archive"));
    await userEvent.click(getByLabelText("Browse"));
    await userEvent.click(getByRole("button", { name: "Open" }));

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|open_volumes", {
      sessionId: expect.any(String),
      path: ARCHIVE_VOLUME,
    });
  });

  it("opens on the directory mode until something else is chosen", async () => {
    const { findByText } = renderForm();

    expect(await findByText("Indexes every archive in the directory for browsing.")).toBeInTheDocument();
  });

  it("comes back on the mode last used, because that is a better guess than the common one", async () => {
    const first = renderForm();

    await userEvent.click(first.getByLabelText("Open archive"));
    first.unmount();

    const { findByText } = renderForm();

    expect(await findByText("Indexes one archive volume for browsing.")).toBeInTheDocument();
  });

  it("keeps each mode's path across a switch", async () => {
    const { getByDisplayValue, getByLabelText } = renderForm();

    mockOpen.mockResolvedValue(ARCHIVES_DIRECTORY);

    await userEvent.click(getByLabelText("Browse"));
    await userEvent.click(getByLabelText("Open archive"));

    mockOpen.mockResolvedValue(ARCHIVE_VOLUME);

    await userEvent.click(getByLabelText("Browse"));

    expect(getByDisplayValue(ARCHIVE_VOLUME)).toBeInTheDocument();

    // Two fields rather than one, so returning to a mode returns to what it was pointed at.
    await userEvent.click(getByLabelText("Open directory"));

    expect(getByDisplayValue(ARCHIVES_DIRECTORY)).toBeInTheDocument();
  });
  it("opens a game folder as a world rather than as a set of volumes", async () => {
    mockOpen.mockResolvedValue(INSTALLATION);

    const { getByLabelText, getByRole, getByText } = renderForm();

    await userEvent.click(getByLabelText("Open installation"));

    expect(
      getByText(
        "Indexes the game the way the engine mounts it: its archives, and the loose gamedata tree standing in " +
          "front of them."
      )
    ).toBeInTheDocument();

    await userEvent.click(getByLabelText("Browse"));
    await userEvent.click(getByRole("button", { name: "Open" }));

    // `auto` rather than `installation`: a folder that declares no `fsgame.ltx` is still read as the game data tree
    // it looks like, instead of being refused for not declaring one.
    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|open_world", {
      sessionId: expect.any(String),
      roots: { asset: null, roots: [{ mode: "auto", path: INSTALLATION }] },
    });
  });

  it("keeps the game folder apart from the archives directory", async () => {
    // A game folder is the parent of the `db` directory the other mode wants, so one shared field would hand each
    // mode the other's answer.
    const { getByDisplayValue, getByLabelText, queryByDisplayValue } = renderForm();

    mockOpen.mockResolvedValue(ARCHIVES_DIRECTORY);

    await userEvent.click(getByLabelText("Browse"));
    await userEvent.click(getByLabelText("Open installation"));

    expect(queryByDisplayValue(ARCHIVES_DIRECTORY)).not.toBeInTheDocument();

    mockOpen.mockResolvedValue(INSTALLATION);

    await userEvent.click(getByLabelText("Browse"));
    await userEvent.click(getByLabelText("Open directory"));

    expect(getByDisplayValue(ARCHIVES_DIRECTORY)).toBeInTheDocument();
  });
});
