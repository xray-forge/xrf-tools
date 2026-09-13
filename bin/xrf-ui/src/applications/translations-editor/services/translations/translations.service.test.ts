import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { EventBus } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";

import { TranslationsService } from "@/applications/translations-editor/services/translations/translations.service";
import { createRoots } from "@/core/assets/lib/roots";
import { TranslationSaveOutcome } from "@/core/ipc/types/xrf-app";
import { TranslationProjectDescriptor } from "@/core/ipc/types/xrf-translation";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity } from "@/core/notifications/lib";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

/** The one file, entry and language every project here holds; which project they came from is what is under test. */
const FILE: string = "st_test.json";
const ID: string = "st_test";
const LANGUAGE: string = "eng";

const FIRST_ROOT: string = "C:\\first";
const SECOND_ROOT: string = "C:\\second";

function createProject(root: string, value: string): TranslationProjectDescriptor {
  return {
    mode: "source",
    roots: createRoots([root]),
    prefix: "translations",
    languages: [LANGUAGE],
    encodings: { [LANGUAGE]: "windows-1251" },
    isEditable: true,
    files: {
      [FILE]: {
        sources: { [LANGUAGE]: { logicalPath: `translations\\${FILE}`, physicalPath: `${root}\\${FILE}` } },
        entries: { [ID]: { [LANGUAGE]: value } },
      },
    },
    findings: [],
  };
}

const PROJECT: TranslationProjectDescriptor = createProject(FIRST_ROOT, "first");
const OTHER_PROJECT: TranslationProjectDescriptor = createProject(SECOND_ROOT, "second");

/** Arrange the state every save starts from: a project open, and one uncommitted edit in it. */
async function openWithEdit(service: TranslationsService): Promise<void> {
  await flowResult(service.openProject(createRoots([FIRST_ROOT]), "source"));

  service.setEdit(FILE, LANGUAGE, ID, "edited");
}

describe("TranslationsService", () => {
  beforeEach(() => {
    setMockInvokeResponses({ ["plugin:translations|get_project"]: mockSessionResponse(() => null) });
  });

  it("saves all dirty files sequentially and excludes another batch", async () => {
    const { service } = mockInjectedService(TranslationsService);
    let finish: (value: TranslationSaveOutcome) => void = noop;
    const answer = new Promise<TranslationSaveOutcome>((resolve) => {
      finish = resolve;
    });
    const write = jest
      .fn()
      .mockImplementationOnce(() => answer)
      .mockImplementation(() => ({ kind: "saved", project: mockSessionSnapshot(PROJECT) }));

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: mockSessionResponse(PROJECT),
      ["plugin:translations|save_file"]: write,
    });

    await openWithEdit(service);
    service.setEdit("second.json", LANGUAGE, ID, "second edit");

    const saving = flowResult(service.saveAll());

    const overlapping = flowResult(service.saveAll());

    expect(write).toHaveBeenCalledTimes(1);
    expect(service.savingFile).toBe(FILE);

    finish({ kind: "saved", project: mockSessionSnapshot(PROJECT) });

    expect(await saving).toBe(true);
    expect(await overlapping).toBe(true);
    expect(write).toHaveBeenNthCalledWith(2, {
      sessionId: expect.any(String),
      file: "second.json",
      edits: { [LANGUAGE]: [{ kind: "set", id: ID, value: "second edit" }] },
    });
    expect(service.dirtyFiles).toEqual([]);
    expect(service.savingFile).toBeNull();
  });

  it.each(["failed", "stale"])("stops a batch at the first %s file", async (outcome) => {
    const { service } = mockInjectedService(TranslationsService);
    const write = jest.fn(() => {
      if (outcome === "failed") {
        throw new Error("write failed");
      }

      return { kind: "stale" };
    });

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: mockSessionResponse(PROJECT),
      ["plugin:translations|save_file"]: write,
    });

    await openWithEdit(service);
    service.setEdit("second.json", LANGUAGE, ID, "second edit");

    expect(await flowResult(service.saveAll())).toBe(false);
    expect(write).toHaveBeenCalledTimes(1);
    expect(service.dirtyFiles).toContain("second.json");
    expect(service.savingFile).toBeNull();
  });

  it("adopts what a save left on disk", async () => {
    const { service } = mockInjectedService(TranslationsService);

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: mockSessionResponse(() => PROJECT),
      ["plugin:translations|save_file"]: () => ({ kind: "saved", project: mockSessionSnapshot(OTHER_PROJECT) }),
    });

    await openWithEdit(service);

    expect(await flowResult(service.saveFile(FILE))).toBe(true);

    expect(mockInvoke).toHaveBeenCalledWith("plugin:translations|save_file", {
      sessionId: expect.any(String),
      file: FILE,
      edits: { [LANGUAGE]: [{ kind: "set", id: ID, value: "edited" }] },
    });
    expect(service.project.value).toEqual(OTHER_PROJECT);
    expect(service.dirtyFiles).toEqual([]);
    expect(service.savingFile).toBeNull();
  });

  it("keeps showing the open project when a save comes back stale", async () => {
    const { service } = mockInjectedService(TranslationsService);

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: mockSessionResponse(() => PROJECT),
      // What the backend answers when the project was replaced while the edits were being written. It withholds the
      // refreshed tree on purpose, and the shown project has to survive that answer untouched.
      ["plugin:translations|save_file"]: () => ({ kind: "stale" }),
    });

    await openWithEdit(service);

    // Reported as a failure so a save of every dirty file stops rather than writing the rest into a project nobody is
    // looking at.
    expect(await flowResult(service.saveFile(FILE))).toBe(false);

    expect(service.project.value).toEqual(PROJECT);
    expect(service.savingFile).toBeNull();
    // The edits did land on disk, so they are not pending work any more.
    expect(service.dirtyFiles).toEqual([]);
  });

  it("leaves the work where it is when a save fails", async () => {
    const { service } = mockInjectedService(TranslationsService);

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: mockSessionResponse(() => PROJECT),
      ["plugin:translations|save_file"]: () => {
        throw new Error("Translations file is no longer in the mounted roots");
      },
    });

    await openWithEdit(service);

    expect(await flowResult(service.saveFile(FILE))).toBe(false);

    expect(service.project.value).toEqual(PROJECT);
    expect(service.dirtyFiles).toEqual([FILE]);
    expect(service.savingFile).toBeNull();
  });

  it("stops marking a file as saving when an open supersedes the save", async () => {
    const { service } = mockInjectedService(TranslationsService);

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: mockSessionResponse(() => PROJECT),
      // A save that never answers, so the open below lands while its write is still in flight.
      ["plugin:translations|save_file"]: () => new Promise(() => undefined),
    });

    await openWithEdit(service);

    const saving: Promise<boolean> = flowResult(service.saveFile(FILE));

    // Cancellation rejects the flow, which is expected here and not what this test is about.
    saving.catch(() => undefined);

    expect(service.savingFile).toBe(FILE);

    // The two share one flow lane, so opening cancels the save rather than queueing behind it. The write itself is
    // already in flight and cannot be recalled; what must not survive is the marker saying this file is being written.
    await flowResult(service.openProject(createRoots([SECOND_ROOT]), "source"));

    expect(service.savingFile).toBeNull();
  });

  it("keeps the project and edits usable after a failed close, then permits a retry", async () => {
    const { service, container } = mockInjectedService(TranslationsService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));
    setMockInvokeResponses({
      ["plugin:translations|open_project"]: mockSessionResponse(PROJECT),
      ["plugin:translations|close_project"]: () => {
        throw "backend refused";
      },
    });

    await openWithEdit(service);

    const previous = service.project.value;

    await service.closeProject();

    expect(service.project.value).toBe(previous);
    expect(service.project.isLoading).toBe(false);
    expect(service.resolveValue(FILE, LANGUAGE, ID)).toBe("edited");
    expect(service.dirtyFiles).toEqual([FILE]);
    expect(notices).toEqual([
      expect.objectContaining({
        details: "backend refused",
        severity: ENotificationSeverity.ERROR,
        title: "Could not close translations project",
      }),
    ]);

    setMockInvokeResponses({});
    await service.closeProject();

    expect(service.project.isLoading).toBe(false);
    expect(service.project.error).toBeNull();
    expect(service.project.value).toBeNull();
    expect(service.edits).toEqual({});
    expect(service.dirtyFiles).toEqual([]);
  });

  it("clears pending edits only after the backend confirms the close", async () => {
    const { service } = mockInjectedService(TranslationsService);
    let finishClose: () => void = noop;
    const answer = new Promise<void>((resolve) => {
      finishClose = resolve;
    });

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: mockSessionResponse(PROJECT),
      ["plugin:translations|close_project"]: () => answer,
    });

    await openWithEdit(service);

    const closing = flowResult(service.closeProject());

    expect(service.project.isLoading).toBe(true);
    expect(service.project.value).toEqual(PROJECT);
    expect(service.dirtyFiles).toEqual([FILE]);

    finishClose();
    await closing;

    expect(service.project.isLoading).toBe(false);
    expect(service.project.error).toBeNull();
    expect(service.project.value).toBeNull();
    expect(service.edits).toEqual({});
    expect(service.savingFile).toBeNull();
  });
});
