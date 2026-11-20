import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { ETranslationsCommand } from "@xrf/ipc/commands/translations";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ETranslationLanguage, TranslationBuildResult } from "@xrf/ipc/types/xrf-translation";

import { clearLocalStorage, setLocalStorageValue } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";
import { TranslationsBuilderService } from "@/tools/translations-builder/services/translations-builder.service";

import { TranslationsBuilderTool } from "./TranslationsBuilderTool";

const SOURCES: string = "C:\\work\\translations";
const OUTPUT: string = "C:\\work\\gamedata\\configs\\text";

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

function renderTool(): RenderResult {
  return renderWithProviders(<TranslationsBuilderTool />, {
    route: "/translations-builder",
    bindings: [TranslationsBuilderService],
  });
}

describe("TranslationsBuilderTool", () => {
  beforeEach(() => {
    clearLocalStorage();
    mockIpcCommands({ [ETranslationsCommand.BUILD_PROJECT]: RESULT });
  });

  it("offers no build until both the sources and the output are named", async () => {
    setLocalStorageValue("xrf.form.translations-builder.sources", SOURCES);

    const { findByRole, getByRole } = renderTool();
    const submit = await findByRole("button", { name: "Build" });

    expect(submit).toBeDisabled();

    fireEvent.change(getByRole("textbox", { name: "Output" }), { target: { value: OUTPUT } });

    await waitFor(() => expect(submit).toBeEnabled());
  });

  it("builds every language, sorted, by default", async () => {
    setLocalStorageValue("xrf.form.translations-builder.sources", SOURCES);
    setLocalStorageValue("xrf.form.translations-builder.destination", OUTPUT);

    const { findByRole, findByText } = renderTool();
    const submit = await findByRole("button", { name: "Build" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText("Built 6 string table(s) from 3 source(s)")).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith(ETranslationsCommand.BUILD_PROJECT, {
      request: {
        roots: { asset: null, roots: [{ path: SOURCES, mode: "auto" }] },
        prefix: null,
        language: ETranslationLanguage.ALL,
        outputDir: OUTPUT,
        isSorted: true,
      },
      jobId: expect.any(String),
      progress: expect.anything(),
    });
  });

  it("builds the remembered language in source order once sorting is off", async () => {
    setLocalStorageValue("xrf.form.translations-builder.sources", SOURCES);
    setLocalStorageValue("xrf.form.translations-builder.destination", OUTPUT);
    setLocalStorageValue("xrf.form.translations-builder.language", ETranslationLanguage.UKRAINIAN);

    const { findByRole, findByText, getByRole } = renderTool();
    const submit = await findByRole("button", { name: "Build" });

    fireEvent.click(getByRole("switch", { name: /Sort ids/ }));
    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText("Built 6 string table(s) from 3 source(s)")).toBeInTheDocument();
    expect(listMockInvokeCalls(ETranslationsCommand.BUILD_PROJECT)).toEqual([
      expect.objectContaining({
        request: expect.objectContaining({ language: ETranslationLanguage.UKRAINIAN, isSorted: false }),
      }),
    ]);
  });

  it("drops the answer of the last build once a field it read changes", async () => {
    setLocalStorageValue("xrf.form.translations-builder.sources", SOURCES);
    setLocalStorageValue("xrf.form.translations-builder.destination", OUTPUT);

    const { findByRole, findByText, getByRole, queryByText } = renderTool();
    const submit = await findByRole("button", { name: "Build" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText("Built 6 string table(s) from 3 source(s)")).toBeInTheDocument();

    fireEvent.click(getByRole("button", { name: "Show parameters" }));
    fireEvent.click(getByRole("switch", { name: /Sort ids/ }));

    expect(queryByText("Built 6 string table(s) from 3 source(s)")).not.toBeInTheDocument();
  });
});
