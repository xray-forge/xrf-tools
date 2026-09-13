import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescriptor } from "@/core/ipc/types/xrf-archive";
import { EPathEntryKind } from "@/core/path/entry-kind";
import { mockArchiveFileDescriptor } from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveFileHeader } from "./ArchiveFileHeader";

const FILE: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\gameplay\\dialogs.xml" });

function renderHeader(): { render: RenderResult; service: ArchivesService } {
  const { service, container } = mockInjectedService(ArchivesService);

  service.selection = { kind: EPathEntryKind.FILE, entry: FILE };

  return { service, render: renderWithProviders(<ArchiveFileHeader entry={FILE} />, { container }) };
}

describe("ArchiveFileHeader", () => {
  it("clears the selection when the file is closed", async () => {
    const { render, service } = renderHeader();

    await userEvent.click(render.getByLabelText("Close file"));

    expect(service.selection).toEqual({ kind: "none" });
  });
});
