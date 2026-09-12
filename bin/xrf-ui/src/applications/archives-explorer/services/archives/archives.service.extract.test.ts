import { beforeEach, describe, expect, it } from "@jest/globals";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/archives.service";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { mockArchiveFileDescriptor, mockArchivesProject } from "@/fixtures/mocks/archive.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";
import { Nullable } from "@/lib/types/general";

const FILE: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

/**
 * Returns the file path from the last file extraction.
 *
 * @param service - Archives service state to inspect.
 * @returns Extracted file path, or null when the last operation was not a file extraction.
 */
function extractedFile(service: ArchivesService): Nullable<string> {
  return service.operation.value?.kind === "extract-file" ? service.operation.value.destination : null;
}

describe("ArchivesService extraction", () => {
  beforeEach(() => {
    setMockInvokeResponses({});
  });

  it("asks the backend for the file by its archived name", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["projectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesProject()));

    await service.extractFile(FILE, "C:\\out\\system.ltx");

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|extract_file", {
      sessionId: expect.any(String),
      name: "configs\\system.ltx",
      destination: "C:\\out\\system.ltx",
    });
    expect(extractedFile(service)).toBe("C:\\out\\system.ltx");
    expect(service.operation.isLoading).toBe(false);
  });

  it("reports a refused extraction instead of staying loading", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["projectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesProject()));

    setMockInvokeResponses({
      ["plugin:archives|extract_file"]: () => {
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

    service["projectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesProject()));

    await service.extractFile(FILE, "C:\\out\\system.ltx");

    service.clearOperation();

    expect(service.operation.value?.kind === "extract-file" ? service.operation.value.destination : null).toBeNull();
    expect(service.operation.error).toBeNull();
  });
});
