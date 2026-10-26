import { beforeEach, describe, expect, it } from "@jest/globals";
import { AsyncState } from "@xrf/async";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveExtractDirectoryResult } from "@xrf/ipc/types/xrf-pack";
import { Nullable } from "@xrf/types";

import {
  mockArchiveExtractDirectoryResult,
  mockArchiveFileDescriptor,
  mockArchivesVolumes,
} from "@/core/archive/fixtures/archive.mocks";
import { EPathEntryKind } from "@/core/path/entry-kind";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { ArchivesService } from "./archives.service";

function getExtractedDirectory(service: ArchivesService): Nullable<ArchiveExtractDirectoryResult> {
  return service.operation.value?.kind === "extract-directory" ? service.operation.value.result : null;
}

describe("ArchivesService directory extraction", () => {
  beforeEach(() => {
    mockIpcCommands({});
  });

  it("sends the directory prefix and destination root", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes()));

    mockIpcCommands({
      [EArchivesCommand.EXTRACT_DIRECTORY]: mockArchiveExtractDirectoryResult({
        prefix: "configs",
        destination: "C:\\out",
        extractedCount: 12,
        size: 4096,
      }),
    });

    await service.extractArchiveDirectory("configs", "C:\\out");

    // The identity and the channel are minted per run, so this names what the caller chose and lets the job's own two
    // arguments be whatever the jobs service made them.
    expect(mockInvoke).toHaveBeenCalledWith(
      EArchivesCommand.EXTRACT_DIRECTORY,
      expect.objectContaining({ request: { sessionId: "fixture-session", prefix: "configs", destination: "C:\\out" } })
    );
    expect(getExtractedDirectory(service)?.extractedCount).toBe(12);
  });

  it("treats the archive root as an empty prefix", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes()));

    service.selectArchiveDirectory("");

    expect(service.selection).toEqual({ kind: EPathEntryKind.DIRECTORY, path: "" });

    await service.extractArchiveDirectory("", "C:\\out");

    expect(mockInvoke).toHaveBeenCalledWith(
      EArchivesCommand.EXTRACT_DIRECTORY,
      expect.objectContaining({ request: { sessionId: "fixture-session", prefix: "", destination: "C:\\out" } })
    );
  });

  it("reports a refused extraction instead of staying loading", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes()));

    mockIpcCommands({
      [EArchivesCommand.EXTRACT_DIRECTORY]: () => {
        throw new Error("destination is read only");
      },
    });

    await expect(service.extractArchiveDirectory("configs", "C:\\out")).rejects.toThrow("read only");

    expect(service.operation.isLoading).toBe(false);
    expect(String(service.operation.error)).toContain("read only");
  });

  it("keeps file and directory selection mutually exclusive", async () => {
    const { service } = mockInjectedService(ArchivesService);

    service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes()));

    service.selectArchiveDirectory("configs");
    expect(service.selection).toEqual({ kind: EPathEntryKind.DIRECTORY, path: "configs" });

    // Both being set at once would leave the content area with two things claiming to be selected.
    const entry = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

    await service.selectArchiveFile(entry);
    expect(service.selection).toEqual({ kind: EPathEntryKind.FILE, entry });
    expect(service.selectedEntry).toEqual(entry);

    service.selectArchiveDirectory("configs");
    expect(service.selection).toEqual({ kind: EPathEntryKind.DIRECTORY, path: "configs" });
    expect(service.selectedEntry).toBeNull();
  });
});
