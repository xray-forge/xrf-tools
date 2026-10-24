import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { mockInvoke, setMockInvokeResponses } from "@xrf/ipc/testing";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { mockArchiveFileDescriptor, mockArchivesVolumes } from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveUnpackDialog } from "./ArchiveUnpackDialog";

async function renderDialog(onClose: () => void): Promise<RenderResult> {
  setMockInvokeResponses({
    ["plugin:archives|restore_subject"]: mockSessionResponse(
      mockArchivesVolumes([mockArchiveFileDescriptor({ name: "configs\\system.ltx" })])
    ),
    ["plugin:system|get_default_output_root"]: "C:\\xrf\\out",
    ["plugin:archives|extract_directory"]: { prefix: "", destination: "", extractedCount: 1, size: 9 },
  });

  const { container, service } = mockInjectedService(ArchivesService);

  const render: RenderResult = renderWithProviders(<ArchiveUnpackDialog isOpen={true} onClose={onClose} />, {
    container,
  });

  await waitFor(() => expect(service.subject.isReady).toBe(true));

  return render;
}

describe("ArchiveUnpackDialog", () => {
  it("unpacks the whole tree straight into the output directory, then closes", async () => {
    const onClose = jest.fn();
    const { getByRole } = await renderDialog(onClose);

    const unpack: HTMLElement = getByRole("button", { name: "Unpack" });

    // The output is seeded from the application's own directory, so the run can start without picking anything.
    await waitFor(() => expect(unpack).toBeEnabled());

    await userEvent.click(unpack);

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith(
        "plugin:archives|extract_directory",
        expect.objectContaining({
          request: { sessionId: "fixture-session", prefix: "", destination: "C:\\xrf\\out\\archives-explorer" },
        })
      )
    );
    expect(onClose).toHaveBeenCalled();
  });
});
