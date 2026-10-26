import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container, EventBus, WireEvent } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { SessionChange, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { ArchiveStatistics } from "@xrf/ipc/types/xrf-archive-stats";

import {
  mockArchiveStatistics,
  mockArchivesWorldSubject,
  mockArchiveWorldEntry,
} from "@/core/archive/fixtures/archive.mocks";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

import { ArchivesService } from "./archives.service";

const RESTORE: string = EArchivesCommand.RESTORE_SUBJECT;

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

function countReads(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === RESTORE).length;
}

function notify(container: Container, change: SessionChange): void {
  container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, change);
}

/**
 * Provisions the explorer, as its application does, and opens a world listing one file; once open, its restore answers
 * a world listing two, at the revision a commit derived it to.
 */
async function openWorld() {
  const { container, service } = mockInjectedService(ArchivesService);
  let sessionId: string = "";

  mockIpcCommands({
    [EArchivesCommand.OPEN_WORLD]: mockSessionResponse((args?: Record<string, unknown>) => {
      sessionId = String(args?.sessionId);

      return mockArchivesWorldSubject([mockArchiveWorldEntry()]);
    }),
    [RESTORE]: () =>
      sessionId
        ? mockSessionSnapshot(
            mockArchivesWorldSubject([mockArchiveWorldEntry(), mockArchiveWorldEntry({ name: "configs\\game.ltx" })]),
            sessionId,
            1
          )
        : null,
  });

  container.provision();
  await flush();
  await service.openWorld({ asset: null, roots: [{ mode: "auto", path: "C:\\game" }] });

  return { container, service, sessionId, reads: countReads() };
}

describe("ArchivesService session changes", () => {
  beforeEach(() => {
    resetMockInvoke();
    mockInvoke.mockClear();
  });

  it("lists the world again once a commit derived it past what is shown", async () => {
    const { container, service, sessionId, reads } = await openWorld();

    expect(service.entries).toHaveLength(1);

    notify(container, { sessionId, revision: 1, root: { kind: "derived" }, staleParts: [], isViewRenewed: false });
    await flush();

    expect(countReads()).toBe(reads + 1);
    expect(service.entries.map((entry) => entry.name)).toEqual(["configs\\system.ltx", "configs\\game.ltx"]);
  });

  it("reads nothing for a change to another opening, one kept, or one it already shows", async () => {
    const { container, service, sessionId, reads } = await openWorld();

    notify(container, {
      sessionId: "other",
      revision: 1,
      root: { kind: "derived" },
      staleParts: [],
      isViewRenewed: false,
    });
    notify(container, { sessionId, revision: 1, root: { kind: "kept" }, staleParts: [], isViewRenewed: false });
    notify(container, { sessionId, revision: 0, root: { kind: "derived" }, staleParts: [], isViewRenewed: false });
    await flush();

    expect(countReads()).toBe(reads);
    expect(service.entries).toHaveLength(1);
  });

  it("drops statistics read from the listing a commit replaced, and reads them again for the new one", async () => {
    const { container, service, sessionId } = await openWorld();
    let finishOlder: (answer: SessionSnapshot<ArchiveStatistics>) => void = noop;

    mockIpcCommands({
      [RESTORE]: () =>
        mockSessionSnapshot(
          mockArchivesWorldSubject([mockArchiveWorldEntry(), mockArchiveWorldEntry({ name: "configs\\game.ltx" })]),
          sessionId,
          1
        ),
      [EArchivesCommand.DESCRIBE_STATISTICS]: () =>
        new Promise((resolve) => {
          finishOlder = resolve;
        }),
    });

    const loading = flowResult(service.loadStatistics());

    notify(container, { sessionId, revision: 1, root: { kind: "derived" }, staleParts: [], isViewRenewed: false });
    await flush();

    expect(service.entries).toHaveLength(2);

    finishOlder(mockSessionSnapshot(mockArchiveStatistics(), sessionId, 0));
    await loading;

    expect(service.statistics.value).toBeNull();
    expect(service.statistics.isLoading).toBe(false);

    mockIpcCommands({
      [EArchivesCommand.DESCRIBE_STATISTICS]: ({ sessionId: id }) =>
        mockSessionSnapshot(mockArchiveStatistics(), id, 1),
    });
    await flowResult(service.loadStatistics());

    expect(service.statistics.value?.overview.total.files).toBe(7);
  });

  it("keeps the world shown and says so where it could not be listed again", async () => {
    const { container, service, sessionId, reads } = await openWorld();
    const notices: Array<INotificationPayload> = [];

    container
      .get(EventBus)
      .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) => notices.push(event.payload!));

    notify(container, {
      sessionId,
      revision: 1,
      root: { kind: "failed", reason: "the volume went" },
      staleParts: [],
      isViewRenewed: false,
    });
    await flush();

    expect(countReads()).toBe(reads);
    expect(service.entries).toHaveLength(1);
    expect(notices).toEqual([
      expect.objectContaining({ details: "the volume went", severity: ENotificationSeverity.WARNING }),
    ]);
  });
});
