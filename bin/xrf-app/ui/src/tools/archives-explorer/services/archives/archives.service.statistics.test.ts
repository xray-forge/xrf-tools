import { describe, expect, it } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveStatistics } from "@xrf/ipc/types/xrf-archive-stats";
import { Nullable } from "@xrf/types";

import { mockArchiveStatistics, mockArchivesVolumes } from "@/core/archive/fixtures/archive.mocks";
import { mockHeldSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { mockSessionRead, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

function getStatisticsCallsCount(): number {
  return mockInvoke.mock.calls.filter((call: Array<unknown>) => call[0] === EArchivesCommand.DESCRIBE_STATISTICS)
    .length;
}

function mockArchivesService(): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  mockHeldSessionRoot(service.subject, mockSessionSnapshot(mockArchivesVolumes()));

  return service;
}

describe("ArchivesService statistics", () => {
  it("describes the open subject once and keeps the answer", async () => {
    mockIpcCommands({ [EArchivesCommand.DESCRIBE_STATISTICS]: mockSessionRead(mockArchiveStatistics()) });

    const service: ArchivesService = mockArchivesService();

    await flowResult(service.loadStatistics());

    const statistics: Nullable<ArchiveStatistics> = service.statistics.value;

    expect(statistics?.overview.total.files).toBe(7);
    expect(getStatisticsCallsCount()).toBe(1);

    // Reopening the dialog must be free: the subject cannot have changed while it is open, and the answer costs a walk
    // of the whole listing.
    await flowResult(service.loadStatistics());

    expect(getStatisticsCallsCount()).toBe(1);
  });

  it("drops the answer when the subject is closed, so the next one is described afresh", async () => {
    mockIpcCommands({ [EArchivesCommand.DESCRIBE_STATISTICS]: mockSessionRead(mockArchiveStatistics()) });

    const service: ArchivesService = mockArchivesService();

    await flowResult(service.loadStatistics());
    await service.closeSubject();

    expect(service.statistics.value).toBeNull();
  });

  it("reports a failed description rather than leaving the dialog blank", async () => {
    mockIpcCommands({
      [EArchivesCommand.DESCRIBE_STATISTICS]: mockSessionRead(() => {
        throw new Error("the session is gone");
      }),
    });

    const service: ArchivesService = mockArchivesService();

    await flowResult(service.loadStatistics());

    expect(service.statistics.value).toBeNull();
    expect(service.statistics.error?.message).toContain("the session is gone");
  });
});
