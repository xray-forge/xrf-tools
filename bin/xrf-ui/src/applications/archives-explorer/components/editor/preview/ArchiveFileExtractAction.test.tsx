import { describe, expect, it, jest } from "@jest/globals";
import * as dialog from "@tauri-apps/plugin-dialog";
import { RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Injectable } from "@wirestate/core";

import { ArchiveFileExtractAction } from "@/applications/archives-explorer/components/editor/preview/ArchiveFileExtractAction";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { mockArchiveFileDescriptor } from "@/fixtures/mocks/archive.mocks";
import { mockInvoke } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const FILE: ArchiveFileDescriptor = mockArchiveFileDescriptor({
  name: "configs\\gameplay\\dialogs.xml",
});

@Injectable()
class TestArchivesService extends ArchivesService {}

function renderAction(): RenderResult {
  return renderWithProviders(<ArchiveFileExtractAction descriptor={FILE} />, {
    bindings: [{ token: ArchivesService, type: "Instance", value: TestArchivesService }],
  });
}

function extractCalls(): Array<unknown> {
  return mockInvoke.mock.calls.filter(([command]) => command === "plugin:archives|extract_file");
}

describe("ArchiveFileExtractAction", () => {
  it("suggests the archived file name so the extension survives", async () => {
    const save = jest.spyOn(dialog, "save").mockResolvedValue("C:\\out\\dialogs.xml");

    const { getByLabelText } = renderAction();

    await userEvent.click(getByLabelText("Extract file"));

    // The archived name is a full path; only its leaf makes sense as a suggested file name, and the
    // filter is what stops the dialog dropping the extension when the name is edited.
    await waitFor(() =>
      expect(save).toHaveBeenCalledWith(
        expect.objectContaining({
          defaultPath: "dialogs.xml",
          filters: [{ name: "XML file", extensions: ["xml"] }],
        })
      )
    );

    save.mockRestore();
  });

  it("writes to the chosen path", async () => {
    const save = jest.spyOn(dialog, "save").mockResolvedValue("C:\\out\\dialogs.xml");

    const { getByLabelText } = renderAction();

    await userEvent.click(getByLabelText("Extract file"));

    await waitFor(() => expect(extractCalls()).toHaveLength(1));
    expect(extractCalls()[0]).toEqual([
      "plugin:archives|extract_file",
      { name: FILE.name, destination: "C:\\out\\dialogs.xml" },
    ]);

    save.mockRestore();
  });

  it("does nothing when the save dialog is cancelled", async () => {
    const save = jest.spyOn(dialog, "save").mockResolvedValue(null);

    const { getByLabelText } = renderAction();

    await userEvent.click(getByLabelText("Extract file"));

    await waitFor(() => expect(save).toHaveBeenCalled());
    expect(extractCalls()).toHaveLength(0);

    save.mockRestore();
  });
});
