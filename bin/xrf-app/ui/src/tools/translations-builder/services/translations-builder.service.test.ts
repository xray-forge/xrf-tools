import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";
import { ETranslationsCommand } from "@xrf/ipc/commands/translations";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { TRANSLATIONS_JOB_BUILD } from "@xrf/ipc/types/xrf-app-translations";
import { ETranslationLanguage, TranslationBuildResult } from "@xrf/ipc/types/xrf-translation";

import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { TranslationsBuilderService } from "@/tools/translations-builder/services/translations-builder.service";

const RESULT: TranslationBuildResult = {
  outcome: "completed",
  duration: 120,
  sources: 3,
  files: 6,
  languages: [
    { language: "eng", files: 3, entries: 40 },
    { language: "rus", files: 3, entries: 40 },
  ],
};

function watchBuilder() {
  const { service, container } = mockInjectedService(TranslationsBuilderService);
  const notices: Array<INotificationPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload as INotificationPayload));

  return { service, container, notices };
}

describe("TranslationsBuilderService", () => {
  beforeEach(() => mockIpcCommands({}));

  it("builds the sources tree with the language, output and ordering asked for", async () => {
    mockIpcCommands({ [ETranslationsCommand.BUILD_PROJECT]: RESULT });

    const { service, notices } = watchBuilder();

    await service.build("C:\\translations", ETranslationLanguage.UKRAINIAN, "C:\\out", false);

    expect(mockInvoke).toHaveBeenCalledWith(ETranslationsCommand.BUILD_PROJECT, {
      request: {
        roots: { asset: null, roots: [{ path: "C:\\translations", mode: "auto" }] },
        prefix: null,
        language: ETranslationLanguage.UKRAINIAN,
        outputDir: "C:\\out",
        isSorted: false,
      },
      jobId: expect.any(String),
      progress: expect.anything(),
    });
    expect(service.operation.result).toEqual(RESULT);
    expect(service.operation.isRunning).toBe(false);
    expect(notices).toEqual([
      expect.objectContaining({
        severity: ENotificationSeverity.SUCCESS,
        title: "Built 6 string table(s) from 3 source(s)",
      }),
    ]);
  });

  it("starts no second build while one runs", async () => {
    const response = deferred<TranslationBuildResult>();

    mockIpcCommands({ [ETranslationsCommand.BUILD_PROJECT]: () => response.promise });

    const { service } = watchBuilder();
    const first = service.build("C:\\translations", ETranslationLanguage.ALL, "C:\\out", true);
    const repeated = service.build("C:\\other", ETranslationLanguage.ENGLISH, "C:\\elsewhere", true);

    expect(service.operation.isRunning).toBe(true);
    expect(listMockInvokeCalls(ETranslationsCommand.BUILD_PROJECT)).toHaveLength(1);

    response.resolve(RESULT);
    await Promise.all([first, repeated]);

    expect(service.operation.result).toEqual(RESULT);
  });

  it("keeps a failure on the operation, reports it, and lets a later build succeed", async () => {
    mockIpcCommands({ [ETranslationsCommand.BUILD_PROJECT]: () => Promise.reject("Output directory is read only") });

    const { service, notices } = watchBuilder();

    await service.build("C:\\translations", ETranslationLanguage.ALL, "C:\\out", true);

    expect(service.operation.error).toBe("Output directory is read only");
    expect(service.operation.result).toBeNull();
    expect(notices).toEqual([
      expect.objectContaining({ severity: ENotificationSeverity.ERROR, title: "Could not build translations" }),
    ]);

    mockIpcCommands({ [ETranslationsCommand.BUILD_PROJECT]: RESULT });
    await service.build("C:\\translations", ETranslationLanguage.ALL, "C:\\out", true);

    expect(service.operation.error).toBeNull();
    expect(service.operation.result).toEqual(RESULT);
  });

  it("reports a stopped build as information rather than as a success", async () => {
    mockIpcCommands({ [ETranslationsCommand.BUILD_PROJECT]: { ...RESULT, outcome: "cancelled" } });

    const { service, notices } = watchBuilder();

    await service.build("C:\\translations", ETranslationLanguage.ALL, "C:\\out", true);

    expect(service.operation.result?.outcome).toBe("cancelled");
    expect(notices).toEqual([
      expect.objectContaining({ severity: ENotificationSeverity.INFO, title: "Stopped building translations" }),
    ]);
  });

  it("publishes the result of a build that settled after the window reloaded", async () => {
    const { service, container } = watchBuilder();

    await container.provision();

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "before-reload",
      kind: TRANSLATIONS_JOB_BUILD,
      result: RESULT,
      error: null,
      conclusion: "completed",
    });

    expect(service.operation.result).toEqual(RESULT);

    container.deprovision();
    container.unbindAll();
  });
});
