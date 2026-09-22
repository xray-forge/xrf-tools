import { describe, expect, it } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/index";
import { ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";
import { mockArchiveStatistics, mockArchivesVolumes } from "@/fixtures/mocks/archive.mocks";
import { mockRestoredSession, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";

function getStatisticsCallsCount(): number {
  return mockInvoke.mock.calls.filter((call: Array<unknown>) => call[0] === "plugin:archives|describe_statistics")
    .length;
}

function mockArchivesService(): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  service["subjectState"] = AsyncState.ready(mockRestoredSession(service, mockSessionSnapshot(mockArchivesVolumes())));

  return service;
}

describe("ArchivesService statistics", () => {
  it("describes the open subject once and keeps the answer", async () => {
    setMockInvokeResponses({ ["plugin:archives|describe_statistics"]: mockArchiveStatistics() });

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

  it("drops the answer when the subject is reset, so the next one is described afresh", async () => {
    setMockInvokeResponses({ ["plugin:archives|describe_statistics"]: mockArchiveStatistics() });

    const service: ArchivesService = mockArchivesService();

    await flowResult(service.loadStatistics());
    service.resetSubject();

    expect(service.statistics.value).toBeNull();
  });

  it("reports a failed description rather than leaving the dialog blank", async () => {
    setMockInvokeResponses({
      ["plugin:archives|describe_statistics"]: () => {
        throw new Error("the session is gone");
      },
    });

    const service: ArchivesService = mockArchivesService();

    await flowResult(service.loadStatistics());

    expect(service.statistics.value).toBeNull();
    expect(service.statistics.error?.message).toContain("the session is gone");
  });
});
