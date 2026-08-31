import { beforeEach, describe, expect, it } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";

import { TranslationsService } from "@/applications/translations-editor/services/translations/translations.service";
import { createRoots } from "@/core/assets/lib/roots";
import { TranslationProjectDescriptor } from "@/core/bindings/types/xrf-translation";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

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
    setMockInvokeResponses({ ["plugin:translations|get_project"]: () => null });
  });

  it("adopts what a save left on disk", async () => {
    const { service } = mockInjectedService(TranslationsService);

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: () => PROJECT,
      ["plugin:translations|save_file"]: () => ({ kind: "saved", project: OTHER_PROJECT }),
    });

    await openWithEdit(service);

    expect(await flowResult(service.saveFile(FILE))).toBe(true);

    expect(mockInvoke).toHaveBeenCalledWith("plugin:translations|save_file", {
      file: FILE,
      edits: { [LANGUAGE]: [{ kind: "set", id: ID, value: "edited" }] },
    });
    expect(service.project.value).toBe(OTHER_PROJECT);
    expect(service.dirtyFiles).toEqual([]);
    expect(service.savingFile).toBeNull();
  });

  it("keeps showing the open project when a save comes back stale", async () => {
    const { service } = mockInjectedService(TranslationsService);

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: () => PROJECT,
      // What the backend answers when the project was replaced while the edits were being written. It withholds the
      // refreshed tree on purpose, and the shown project has to survive that answer untouched.
      ["plugin:translations|save_file"]: () => ({ kind: "stale" }),
    });

    await openWithEdit(service);

    // Reported as a failure so a save of every dirty file stops rather than writing the rest into a project nobody is
    // looking at.
    expect(await flowResult(service.saveFile(FILE))).toBe(false);

    expect(service.project.value).toBe(PROJECT);
    expect(service.savingFile).toBeNull();
    // The edits did land on disk, so they are not pending work any more.
    expect(service.dirtyFiles).toEqual([]);
  });

  it("leaves the work where it is when a save fails", async () => {
    const { service } = mockInjectedService(TranslationsService);

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: () => PROJECT,
      ["plugin:translations|save_file"]: () => {
        throw new Error("Translations file is no longer in the mounted roots");
      },
    });

    await openWithEdit(service);

    expect(await flowResult(service.saveFile(FILE))).toBe(false);

    expect(service.project.value).toBe(PROJECT);
    expect(service.dirtyFiles).toEqual([FILE]);
    expect(service.savingFile).toBeNull();
  });

  it("stops marking a file as saving when an open supersedes the save", async () => {
    const { service } = mockInjectedService(TranslationsService);

    setMockInvokeResponses({
      ["plugin:translations|open_project"]: () => PROJECT,
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
});
