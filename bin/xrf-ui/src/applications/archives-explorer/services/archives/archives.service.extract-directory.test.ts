import { beforeEach, describe, expect, it } from "@jest/globals";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/archives.service";
import { ArchiveExtractDirectoryResult } from "@/core/bindings/types/xrf-pack";
import { mockArchiveFileDescriptor } from "@/fixtures/mocks/archive.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

/**
 * Returns the result from the last directory extraction.
 *
 * @param service - Archives service state to inspect.
 * @returns Directory extraction result, or null when the last operation was not a directory extraction.
 */
function extractedDirectory(service: ArchivesService): Nullable<ArchiveExtractDirectoryResult> {
  return service.operation.value?.kind === "extract-directory" ? service.operation.value.result : null;
}

describe("ArchivesService directory extraction", () => {
  beforeEach(() => {
    setMockInvokeResponses({});
  });

  it("sends the directory prefix and destination root", async () => {
    const { service } = mockInjectedService(ArchivesService);

    setMockInvokeResponses({
      ["plugin:archives|extract_directory"]: {
        prefix: "configs",
        destination: "C:\\out",
        extractedCount: 12,
        size: 4096,
      },
    });

    await service.extractArchiveDirectory("configs", "C:\\out");

    // The identity and the channel are minted per run, so this names what the caller chose and lets the job's own two
    // arguments be whatever the jobs service made them.
    expect(mockInvoke).toHaveBeenCalledWith(
      "plugin:archives|extract_directory",
      expect.objectContaining({ prefix: "configs", destination: "C:\\out" })
    );
    expect(extractedDirectory(service)?.extractedCount).toBe(12);
  });

  it("treats the archive root as an empty prefix", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service.selectArchiveDirectory("");

    expect(service.selectedDirectory).toBe("");

    await service.extractArchiveDirectory("", "C:\\out");

    expect(mockInvoke).toHaveBeenCalledWith(
      "plugin:archives|extract_directory",
      expect.objectContaining({ prefix: "", destination: "C:\\out" })
    );
  });

  it("reports a refused extraction instead of staying loading", async () => {
    const { service } = mockInjectedService(ArchivesService);

    setMockInvokeResponses({
      ["plugin:archives|extract_directory"]: () => {
        throw new Error("destination is read only");
      },
    });

    await expect(service.extractArchiveDirectory("configs", "C:\\out")).rejects.toThrow("read only");

    expect(service.operation.isLoading).toBe(false);
    expect(String(service.operation.error)).toContain("read only");
  });

  it("keeps file and directory selection mutually exclusive", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service.selectArchiveDirectory("configs");
    expect(service.selectedDirectory).toBe("configs");

    // Both being set at once would leave the content area with two things claiming to be selected.
    await service.selectArchiveFile(mockArchiveFileDescriptor({ name: "configs\\system.ltx" }));
    expect(service.selectedDirectory).toBeNull();
    expect(service.selectedFile).not.toBeNull();

    service.selectArchiveDirectory("configs");
    expect(service.selectedFile).toBeNull();
  });
});
