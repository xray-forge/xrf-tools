import { beforeEach, describe, expect, it } from "@jest/globals";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";

import { mockArchiveFileDescriptor, mockArchivesVolumes } from "@/core/archive/fixtures/archive.mocks";
import { mockHeldSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { ArchivesService } from "./archives.service";

const FILE: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

function mockService(): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  mockHeldSessionRoot(service.subject, mockSessionSnapshot(mockArchivesVolumes()));

  return service;
}

describe("ArchivesService extraction", () => {
  beforeEach(() => {
    mockIpcCommands({});
  });

  it("asks the backend for the file by its archived name", async () => {
    const service: ArchivesService = mockService();

    await service.extractFile(FILE, "C:\\out\\system.ltx");

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.EXTRACT_FILE, {
      sessionId: expect.any(String),
      name: "configs\\system.ltx",
      destination: "C:\\out\\system.ltx",
    });
    expect(service.fileExtraction.value).toBe("C:\\out\\system.ltx");
    expect(service.fileExtraction.isLoading).toBe(false);
  });

  it("reports a refused extraction instead of staying loading", async () => {
    const service: ArchivesService = mockService();

    mockIpcCommands({
      [EArchivesCommand.EXTRACT_FILE]: () => {
        throw new Error("destination is read only");
      },
    });

    await service.extractFile(FILE, "C:\\out\\system.ltx");

    // Left loading, the header's extract control stays disabled with no way back.
    expect(service.fileExtraction.isLoading).toBe(false);
    expect(String(service.fileExtraction.error)).toContain("read only");
  });

  it("clears a reported outcome", async () => {
    const service: ArchivesService = mockService();

    await service.extractFile(FILE, "C:\\out\\system.ltx");

    service.clearExtractions();

    expect(service.fileExtraction.value).toBeNull();
    expect(service.fileExtraction.error).toBeNull();
  });
});
