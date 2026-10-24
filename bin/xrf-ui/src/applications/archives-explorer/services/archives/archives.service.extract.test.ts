import { beforeEach, describe, expect, it } from "@jest/globals";
import { AsyncState } from "@xrf/async";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";
import { Nullable } from "@xrf/types";

import { mockArchiveFileDescriptor, mockArchivesVolumes } from "@/fixtures/mocks/archive.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { ArchivesService } from "./archives.service";

const FILE: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

function getExtractedFile(service: ArchivesService): Nullable<string> {
  return service.operation.value?.kind === "extract-file" ? service.operation.value.destination : null;
}

describe("ArchivesService extraction", () => {
  beforeEach(() => {
    mockIpcCommands({});
  });

  it("asks the backend for the file by its archived name", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes()));

    await service.extractFile(FILE, "C:\\out\\system.ltx");

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.EXTRACT_FILE, {
      sessionId: expect.any(String),
      name: "configs\\system.ltx",
      destination: "C:\\out\\system.ltx",
    });
    expect(getExtractedFile(service)).toBe("C:\\out\\system.ltx");
    expect(service.operation.isLoading).toBe(false);
  });

  it("reports a refused extraction instead of staying loading", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes()));

    mockIpcCommands({
      [EArchivesCommand.EXTRACT_FILE]: () => {
        throw new Error("destination is read only");
      },
    });

    await expect(service.extractFile(FILE, "C:\\out\\system.ltx")).rejects.toThrow("read only");

    // Left loading, the header's extract control stays disabled with no way back.
    expect(service.operation.isLoading).toBe(false);
    expect(String(service.operation.error)).toContain("read only");
  });

  it("clears a reported outcome", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes()));

    await service.extractFile(FILE, "C:\\out\\system.ltx");

    service.clearOperation();

    expect(service.operation.value?.kind === "extract-file" ? service.operation.value.destination : null).toBeNull();
    expect(service.operation.error).toBeNull();
  });
});
