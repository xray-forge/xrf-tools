import { describe, expect, it, jest } from "@jest/globals";
import * as dialog from "@tauri-apps/plugin-dialog";
import { RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Injectable } from "@wirestate/core";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";

import { mockArchiveFileDescriptor, mockArchivesVolumes } from "@/core/archive/fixtures/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

import { ArchiveFileExtractAction } from "./ArchiveFileExtractAction";

const FILE: ArchiveFileDescriptor = mockArchiveFileDescriptor({
  name: "configs\\gameplay\\dialogs.xml",
});

@Injectable()
class TestArchivesService extends ArchivesService {}

function renderAction(): RenderResult {
  mockIpcCommands({ [EArchivesCommand.RESTORE_SUBJECT]: mockSessionResponse(mockArchivesVolumes()) });

  return renderWithProviders(<ArchiveFileExtractAction entry={FILE} />, {
    bindings: [{ token: ArchivesService, type: "Instance", value: TestArchivesService }],
  });
}

function extractCalls(): Array<unknown> {
  return mockInvoke.mock.calls.filter(([command]) => command === EArchivesCommand.EXTRACT_FILE);
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
      EArchivesCommand.EXTRACT_FILE,
      { sessionId: "fixture-session", name: FILE.name, destination: "C:\\out\\dialogs.xml" },
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
