import { beforeEach, describe, expect, it } from "@jest/globals";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";

import {
  mockArchiveExtractDirectoryResult,
  mockArchiveFileDescriptor,
  mockArchivesVolumes,
} from "@/core/archive/fixtures/archive.mocks";
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

describe("ArchivesService directory extraction", () => {
  beforeEach(() => {
    mockIpcCommands({});
  });

  it("sends the directory prefix and destination root", async () => {
    const service: ArchivesService = mockService();

    mockIpcCommands({
      [EArchivesCommand.EXTRACT_DIRECTORY]: mockArchiveExtractDirectoryResult({
        prefix: "configs",
        destination: "C:\\out",
        extractedCount: 12,
        size: 4096,
      }),
    });

    await service.extractDirectory("configs", "C:\\out");

    // The identity and the channel are minted per run, so this names what the caller chose and lets the job's own two
    // arguments be whatever the jobs service made them.
    expect(mockInvoke).toHaveBeenCalledWith(
      EArchivesCommand.EXTRACT_DIRECTORY,
      expect.objectContaining({ request: { sessionId: "fixture-session", prefix: "configs", destination: "C:\\out" } })
    );
    expect(service.extraction.result?.extractedCount).toBe(12);
  });

  it("treats the archive root as an empty prefix", async () => {
    const service: ArchivesService = mockService();

    await service.extractDirectory("", "C:\\out");

    expect(mockInvoke).toHaveBeenCalledWith(
      EArchivesCommand.EXTRACT_DIRECTORY,
      expect.objectContaining({ request: { sessionId: "fixture-session", prefix: "", destination: "C:\\out" } })
    );
  });

  it("reports a refused extraction instead of staying loading", async () => {
    const service: ArchivesService = mockService();

    mockIpcCommands({
      [EArchivesCommand.EXTRACT_DIRECTORY]: () => {
        throw new Error("destination is read only");
      },
    });

    await service.extractDirectory("configs", "C:\\out");

    expect(service.extraction.isRunning).toBe(false);
    expect(service.extraction.error).toContain("read only");
  });

  it("starts no directory while a file is written, as one write runs at a time", () => {
    const service: ArchivesService = mockService();

    mockIpcCommands({ [EArchivesCommand.EXTRACT_FILE]: () => new Promise(() => {}) });

    void service.extractFile(FILE, "C:\\out\\system.ltx");
    void service.extractDirectory("configs", "C:\\out");

    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.EXTRACT_DIRECTORY, expect.anything());
  });
});
