import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container, EventBus, WireEvent } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { ETranslationsCommand } from "@xrf/ipc/commands/translations";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { IpcCommandMocks, mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { SessionChange, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { ETranslationSaveOutcome } from "@xrf/ipc/types/xrf-app-translations";
import { TranslationProjectDescriptor } from "@xrf/ipc/types/xrf-translation";

import { createRoots } from "@/core/assets/lib/roots";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { TranslationsService } from "./translations.service";

const FILE: string = "st_test.json";
const ID: string = "st_test";
const LANGUAGE: string = "eng";
const ROOT: string = "C:\\project";
const GET_PROJECT: string = ETranslationsCommand.RESTORE_PROJECT;

function createProject(value: string): TranslationProjectDescriptor {
  return {
    mode: "source",
    roots: createRoots([ROOT]),
    prefix: "translations",
    languages: [LANGUAGE],
    encodings: { [LANGUAGE]: "windows-1251" },
    isEditable: true,
    files: {
      [FILE]: {
        sources: { [LANGUAGE]: { logicalPath: `translations\\${FILE}`, physicalPath: `${ROOT}\\${FILE}` } },
        entries: { [ID]: { [LANGUAGE]: value } },
      },
    },
    findings: [],
  };
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

function countReads(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === GET_PROJECT).length;
}

interface IOpenedProject {
  container: Container;
  service: TranslationsService;
  sessionId: string;
}

/** Provisions the editor, as its application does, and opens a project whose one cell reads `opened`. */
async function openProject(responses: IpcCommandMocks = {}): Promise<IOpenedProject> {
  const { container, service } = mockInjectedService(TranslationsService);
  let sessionId: string = "";

  mockIpcCommands({
    [GET_PROJECT]: null,
    [ETranslationsCommand.OPEN_PROJECT]: mockSessionResponse((args?: Record<string, unknown>) => {
      sessionId = (args?.request as { sessionId: string }).sessionId;

      return createProject("opened");
    }),
    ...responses,
  });

  container.provision();
  await flush();
  await flowResult(service.openProject(createRoots([ROOT]), "source"));

  return { container, service, sessionId };
}

function notify(container: Container, change: SessionChange): void {
  container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, change);
}

function derived(sessionId: string, revision: number): SessionChange {
  return { sessionId, revision, root: { kind: "derived" }, staleParts: [], isViewRenewed: true };
}

describe("TranslationsService changes", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("reads the project again once a change derives it, showing an edit saved elsewhere", async () => {
    const { container, service, sessionId } = await openProject();
    const before: number = countReads();

    mockIpcCommands({ [GET_PROJECT]: mockSessionSnapshot(createProject("saved elsewhere"), sessionId, 1) });
    notify(container, derived(sessionId, 1));
    await flush();

    expect(countReads() - before).toBe(1);
    expect(service.resolveValue(FILE, LANGUAGE, ID)).toBe("saved elsewhere");
    expect(service.isStale).toBe(false);
  });

  it("reads nothing again for a change to another session, or one that kept the project", async () => {
    const { container, service, sessionId } = await openProject();
    const before: number = countReads();

    notify(container, derived("another", 1));
    notify(container, { sessionId, revision: 1, root: { kind: "kept" }, staleParts: [], isViewRenewed: false });
    await flush();

    expect(countReads()).toBe(before);
    expect(service.resolveValue(FILE, LANGUAGE, ID)).toBe("opened");
    expect(service.isStale).toBe(false);
  });

  it("drops a read answering an older revision than the one shown", async () => {
    const { container, service, sessionId } = await openProject();
    const reads: Array<(snapshot: SessionSnapshot<TranslationProjectDescriptor>) => void> = [];

    mockIpcCommands({ [GET_PROJECT]: () => new Promise((resolve) => reads.push(resolve)) });

    notify(container, derived(sessionId, 1));
    notify(container, derived(sessionId, 2));
    await flush();

    expect(reads).toHaveLength(2);
    expect(service.isStale).toBe(true);

    reads[1](mockSessionSnapshot(createProject("second"), sessionId, 2));
    await flush();
    reads[0](mockSessionSnapshot(createProject("first"), sessionId, 1));
    await flush();

    expect(service.resolveValue(FILE, LANGUAGE, ID)).toBe("second");
    expect(service.isStale).toBe(false);
  });

  it("reads nothing again for the change its own save made", async () => {
    const { container, service, sessionId } = await openProject({
      [ETranslationsCommand.SAVE_FILE]: (args?: Record<string, unknown>) => ({
        kind: ETranslationSaveOutcome.SAVED,
        project: mockSessionSnapshot(createProject("saved"), args?.sessionId as string, 1),
      }),
    });
    const before: number = countReads();

    service.setEdit(FILE, LANGUAGE, ID, "saved");

    expect(await flowResult(service.saveFile(FILE))).toBe(true);

    notify(container, derived(sessionId, 1));
    await flush();

    expect(countReads()).toBe(before);
    expect(service.resolveValue(FILE, LANGUAGE, ID)).toBe("saved");
  });

  it("keeps the project shown where it could not be derived again, and says why", async () => {
    const { container, service, sessionId } = await openProject();
    const notices: Array<INotificationPayload> = [];

    container
      .get(EventBus)
      .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
        notices.push(event.payload as INotificationPayload)
      );

    notify(container, {
      sessionId,
      revision: 0,
      root: { kind: "failed", reason: "st_test.json: bad JSON" },
      staleParts: [],
      isViewRenewed: false,
    });
    await flush();

    expect(service.resolveValue(FILE, LANGUAGE, ID)).toBe("opened");
    expect(notices).toEqual([
      expect.objectContaining({ details: "st_test.json: bad JSON", severity: ENotificationSeverity.WARNING }),
    ]);
  });
});
