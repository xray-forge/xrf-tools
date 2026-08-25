import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { open } from "@tauri-apps/plugin-dialog";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { ArchivesEditorOpenForm } from "@/applications/archives-explorer/components/ArchivesEditorOpenForm";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import { ProjectService } from "@/core/settings/services/project";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

const PROJECT_PATH: string = "C:\\projects\\xrf";
const ARCHIVES_DIRECTORY: string = "C:\\game\\database";
const ARCHIVE_VOLUME: string = "C:\\downloads\\gamedata.db0";

describe("ArchivesEditorOpenForm", () => {
  beforeEach(() => {
    window.localStorage.clear();

    setMockInvokeResponses({
      ["plugin:archives|get_project"]: null,
      ["plugin:archives|open_project"]: null,
    });
  });

  /**
   * The mode defaults from the project path, which the root container resolves long before this form mounts. A test
   * container is provisioned as the form renders, so the path is set on the service rather than awaited.
   */
  function renderForm(projectPath: Nullable<string> = PROJECT_PATH): RenderResult {
    const container: Container = mockContainer([AssetService, ProjectService, ArchivesService]);

    container.get(ProjectService).setXrfProjectPath(projectPath);

    return renderWithProviders(<ArchivesEditorOpenForm />, { route: "/archives-explorer", container });
  }

  it("asks for a directory in directory mode", async () => {
    jest.mocked(open).mockResolvedValue(ARCHIVES_DIRECTORY);

    const { getByLabelText, getByText } = renderForm();

    expect(getByText("Indexes every archive in the directory for browsing. Nothing is written.")).toBeInTheDocument();

    await userEvent.click(getByLabelText("Browse"));

    expect(open).toHaveBeenCalledWith({
      title: "Select archives directory",
      filters: undefined,
      directory: true,
    });
  });

  it("asks for a file, filtered to volumes, in archive mode", async () => {
    jest.mocked(open).mockResolvedValue(ARCHIVE_VOLUME);

    const { getByLabelText, getByText } = renderForm();

    await userEvent.click(getByLabelText("Open archive"));

    expect(getByText("Indexes one archive volume for browsing. Nothing is written.")).toBeInTheDocument();

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
    jest.mocked(open).mockResolvedValue(ARCHIVE_VOLUME);

    const { getByLabelText, getByRole } = renderForm();

    await userEvent.click(getByLabelText("Open archive"));
    await userEvent.click(getByLabelText("Browse"));
    await userEvent.click(getByRole("button", { name: "Open" }));

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|open_project", { path: ARCHIVE_VOLUME });
  });

  it("defaults to a volume when no project names a directory to browse", () => {
    const { getByText } = renderForm(null);

    expect(getByText("Indexes one archive volume for browsing. Nothing is written.")).toBeInTheDocument();
  });

  it("keeps each mode's path across a switch", async () => {
    const { getByDisplayValue, getByLabelText } = renderForm();

    jest.mocked(open).mockResolvedValue(ARCHIVES_DIRECTORY);
    await userEvent.click(getByLabelText("Browse"));

    await userEvent.click(getByLabelText("Open archive"));

    jest.mocked(open).mockResolvedValue(ARCHIVE_VOLUME);
    await userEvent.click(getByLabelText("Browse"));

    expect(getByDisplayValue(ARCHIVE_VOLUME)).toBeInTheDocument();

    // Two fields rather than one, so returning to a mode returns to what it was pointed at.
    await userEvent.click(getByLabelText("Open directory"));

    expect(getByDisplayValue(ARCHIVES_DIRECTORY)).toBeInTheDocument();
  });
});
