import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { TranslationsEditorApplication } from "@/applications/translations-editor/TranslationsEditorApplication";
import { TranslationProjectDescriptor } from "@/core/bindings/types/xrf-translation";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

const FIRST_FILE: string = "configs\\text\\ui_st_first.xml";
const SECOND_FILE: string = "configs\\text\\ui_st_second.xml";

/** Both files carry `shared_id`, which is what makes an answer landing on the wrong one visible. */
const PROJECT: TranslationProjectDescriptor = {
  mode: "gamedata",
  roots: { asset: null, roots: [{ mode: "auto", path: "C:\\game" }] },
  prefix: "configs\\text",
  languages: ["eng", "rus"],
  encodings: { eng: "windows-1250", rus: "windows-1251" },
  isEditable: true,
  files: {
    [FIRST_FILE]: {
      sources: {
        eng: { logicalPath: "configs\\text\\eng\\ui_st_first.xml", physicalPath: "C:/game/eng/first.xml" },
        rus: { logicalPath: "configs\\text\\rus\\ui_st_first.xml", physicalPath: "C:/game/rus/first.xml" },
      },
      entries: { shared_id: { eng: "First english", rus: "First russian" } },
    },
    [SECOND_FILE]: {
      sources: {
        eng: { logicalPath: "configs\\text\\eng\\ui_st_second.xml", physicalPath: "C:/game/eng/second.xml" },
        rus: { logicalPath: "configs\\text\\rus\\ui_st_second.xml", physicalPath: "C:/game/rus/second.xml" },
      },
      entries: { shared_id: { eng: "Second english", rus: "Second russian" } },
    },
  },
  findings: [],
};

/** A validation the backend has been asked for and has not answered yet. */
interface IPendingValidation {
  language: string;
  text: string;
  answer: (error: Nullable<string>) => void;
}

/** The rendered workspace, with the gestures this suite drives it by bound to its own queries. */
interface ITranslationsEditorView extends RenderResult {
  /** Types over a cell and commits it, which is what asks for a validation. */
  editCell: (from: string, to: string) => Promise<void>;
  selectFile: (file: string) => Promise<void>;
  selectTarget: (language: string) => Promise<void>;
}

const pending: Array<IPendingValidation> = [];

function renderEditor(): ITranslationsEditorView {
  const view: RenderResult = renderWithProviders(<TranslationsEditorApplication />, {
    bindings: [TranslationsService],
    route: "/translations-editor",
  });

  return {
    ...view,
    editCell: async (from: string, to: string): Promise<void> => {
      await userEvent.dblClick(await view.findByRole("gridcell", { name: from }));

      const input: HTMLElement = await view.findByDisplayValue(from);

      await userEvent.clear(input);
      await userEvent.type(input, `${to}{Enter}`);
    },
    selectFile: (file: string): Promise<void> => userEvent.click(view.getByText(file)),
    selectTarget: async (language: string): Promise<void> => {
      await userEvent.click(view.getByLabelText("Target"));
      await userEvent.click(await view.findByRole("option", { name: `${language} · ${PROJECT.encodings[language]}` }));
    },
  };
}

async function answer(index: number, error: Nullable<string>): Promise<void> {
  await act(async () => pending[index].answer(error));
}

describe("deferred translation validation", () => {
  beforeEach(() => {
    pending.length = 0;

    setMockInvokeResponses({
      ["plugin:translations|get_project"]: PROJECT,
      ["plugin:translations|close_project"]: undefined,
      // Answered by the test rather than by the mock: what this suite is about is when an answer lands,
      // not what it says.
      ["plugin:translations|validate_text"]: (parameters?: Record<string, unknown>) =>
        new Promise<Nullable<string>>((resolve) =>
          pending.push({
            language: parameters?.language as string,
            text: parameters?.text as string,
            answer: resolve,
          })
        ),
    });
  });

  it("reports what the target language cannot hold on the cell it was typed in", async () => {
    const { editCell, findByTitle } = renderEditor();

    await editCell("First russian", "Zażółć");
    await answer(0, "Cannot encode 'ż'");

    expect(pending[0].language).toBe("rus");
    expect(await findByTitle("Cannot encode 'ż'")).toBeInTheDocument();
  });

  it("keeps an answer about a replaced value from landing on the one that replaced it", async () => {
    const { editCell, queryByTitle } = renderEditor();

    await editCell("First russian", "Zażółć");
    await editCell("Zażółć", "Zazolc");

    expect(pending).toHaveLength(2);

    // Out of order on purpose: the second edit is answered first, so the first answer arrives against a
    // value that is no longer in the cell.
    await answer(1, null);
    await answer(0, "Cannot encode 'ż'");

    expect(queryByTitle("Cannot encode 'ż'")).not.toBeInTheDocument();
  });

  it("keeps an answer about the previous target language off the one now selected", async () => {
    const { editCell, findByRole, queryByTitle, selectTarget } = renderEditor();

    await editCell("First russian", "Zażółć");
    await selectTarget("eng");
    await answer(0, "Cannot encode 'ż'");

    expect(await findByRole("columnheader", { name: "Target · eng" })).toBeInTheDocument();
    expect(queryByTitle("Cannot encode 'ż'")).not.toBeInTheDocument();
  });

  it("keeps an answer about another file off a cell that only shares its id", async () => {
    const { editCell, findByRole, queryByTitle, selectFile } = renderEditor();

    await editCell("First russian", "Zażółć");
    await selectFile(SECOND_FILE);
    await answer(0, "Cannot encode 'ż'");

    expect(await findByRole("gridcell", { name: "Second russian" })).toBeInTheDocument();
    expect(queryByTitle("Cannot encode 'ż'")).not.toBeInTheDocument();
  });

  it("shows the error again on returning to the file whose edit is still unwritten", async () => {
    const { editCell, findByTitle, selectFile } = renderEditor();

    await editCell("First russian", "Zażółć");
    await selectFile(SECOND_FILE);
    await answer(0, "Cannot encode 'ż'");

    await selectFile(FIRST_FILE);

    // The edit is still pending in that file, so the reason it cannot be written is still true.
    expect(await findByTitle("Cannot encode 'ż'")).toBeInTheDocument();
  });
});
