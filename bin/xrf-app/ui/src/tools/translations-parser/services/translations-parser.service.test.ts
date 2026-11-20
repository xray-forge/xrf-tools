import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";
import { ETranslationsCommand } from "@xrf/ipc/commands/translations";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { TRANSLATIONS_JOB_PARSE } from "@xrf/ipc/types/xrf-app-translations";
import { ETranslationLanguage, TranslationParseResult } from "@xrf/ipc/types/xrf-translation";

import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import {
  mockTranslationParseCensus,
  mockTranslationParseResult,
} from "@/tools/translations-parser/fixtures/translation-parse.mocks";
import { TranslationsParserService } from "@/tools/translations-parser/services/translations-parser.service";

const RESULT: TranslationParseResult = mockTranslationParseResult({
  census: mockTranslationParseCensus({ filesRead: 4, filesCreated: 1, filesUpdated: 2, entriesRead: 80 }),
});

function watchParser() {
  const { service, container } = mockInjectedService(TranslationsParserService);
  const notices: Array<INotificationPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload as INotificationPayload));

  return { service, container, notices };
}

describe("TranslationsParserService", () => {
  beforeEach(() => mockIpcCommands({}));

  it.each([
    ["an import", false],
    ["a preview", true],
  ])("reads the source through its containing installation for %s", async (_, isDryRun) => {
    mockIpcCommands({ [ETranslationsCommand.PARSE_PROJECT]: { ...RESULT, isDryRun } });

    const { service } = watchParser();

    await service.parse("C:\\game\\gamedata", ETranslationLanguage.RUSSIAN, "C:\\out", true, isDryRun);

    expect(mockInvoke).toHaveBeenCalledWith(ETranslationsCommand.PARSE_PROJECT, {
      request: {
        roots: { asset: null, roots: [{ path: "C:\\game\\gamedata", mode: "containingInstallation" }] },
        language: ETranslationLanguage.RUSSIAN,
        prefix: null,
        outputDir: "C:\\out",
        file: null,
        isOverwrite: true,
        isDryRun,
      },
      jobId: expect.any(String),
      progress: expect.anything(),
    });
    expect(service.operation.result?.isDryRun).toBe(isDryRun);
    expect(service.operation.isRunning).toBe(false);
  });

  it("reports a written import by the tables it read", async () => {
    mockIpcCommands({ [ETranslationsCommand.PARSE_PROJECT]: RESULT });

    const { service, notices } = watchParser();

    await service.parse("C:\\game", ETranslationLanguage.ENGLISH, "C:\\out", false, false);

    expect(notices).toEqual([
      expect.objectContaining({ severity: ENotificationSeverity.SUCCESS, title: "Imported 4 table(s)" }),
    ]);
  });

  it("reports a preview by what it would change", async () => {
    mockIpcCommands({ [ETranslationsCommand.PARSE_PROJECT]: { ...RESULT, isDryRun: true } });

    const { service, notices } = watchParser();

    await service.parse("C:\\game", ETranslationLanguage.ENGLISH, "C:\\out", false, true);

    expect(notices).toEqual([
      expect.objectContaining({ severity: ENotificationSeverity.INFO, title: "Import would change 3 file(s)" }),
    ]);
  });

  it("starts no second import while one runs, a preview included", async () => {
    const response = deferred<TranslationParseResult>();

    mockIpcCommands({ [ETranslationsCommand.PARSE_PROJECT]: () => response.promise });

    const { service } = watchParser();
    const first = service.parse("C:\\game", ETranslationLanguage.ENGLISH, "C:\\out", false, false);
    const preview = service.parse("C:\\game", ETranslationLanguage.ENGLISH, "C:\\out", false, true);

    expect(service.operation.isRunning).toBe(true);
    expect(listMockInvokeCalls(ETranslationsCommand.PARSE_PROJECT)).toHaveLength(1);

    response.resolve(RESULT);
    await Promise.all([first, preview]);

    expect(service.operation.result).toEqual(RESULT);
  });

  it("keeps a failure on the operation, reports it, and lets a later import succeed", async () => {
    mockIpcCommands({ [ETranslationsCommand.PARSE_PROJECT]: () => Promise.reject("No string tables found") });

    const { service, notices } = watchParser();

    await service.parse("C:\\game", ETranslationLanguage.ENGLISH, "C:\\out", false, false);

    expect(service.operation.error).toBe("No string tables found");
    expect(notices).toEqual([
      expect.objectContaining({
        severity: ENotificationSeverity.ERROR,
        title: "Could not import translations",
        details: "C:\\out\nNo string tables found",
      }),
    ]);

    mockIpcCommands({ [ETranslationsCommand.PARSE_PROJECT]: RESULT });
    await service.parse("C:\\game", ETranslationLanguage.ENGLISH, "C:\\out", false, false);

    expect(service.operation.error).toBeNull();
    expect(service.operation.result).toEqual(RESULT);
  });

  it("reports a stopped import as information", async () => {
    mockIpcCommands({ [ETranslationsCommand.PARSE_PROJECT]: { ...RESULT, outcome: "cancelled" } });

    const { service, notices } = watchParser();

    await service.parse("C:\\game", ETranslationLanguage.ENGLISH, "C:\\out", false, false);

    expect(notices).toEqual([
      expect.objectContaining({ severity: ENotificationSeverity.INFO, title: "Stopped importing translations" }),
    ]);
  });

  it("publishes the result of an import that settled after the window reloaded", async () => {
    const { service, container } = watchParser();

    await container.provision();

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "before-reload",
      kind: TRANSLATIONS_JOB_PARSE,
      result: RESULT,
      error: null,
      conclusion: "completed",
    });

    expect(service.operation.result).toEqual(RESULT);

    container.deprovision();
    container.unbindAll();
  });
});
