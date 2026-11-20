import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { ETranslationsCommand } from "@xrf/ipc/commands/translations";
import { listMockInvokeCalls, mockIpcCommands } from "@xrf/ipc/testing";
import { TranslationsParseRequest } from "@xrf/ipc/types/xrf-app-translations";
import { ETranslationLanguage, TranslationParseResult } from "@xrf/ipc/types/xrf-translation";

import { clearLocalStorage, setLocalStorageValue } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";
import {
  mockTranslationParseCensus,
  mockTranslationParseResult,
} from "@/tools/translations-parser/fixtures/translation-parse.mocks";
import { TranslationsParserService } from "@/tools/translations-parser/services/translations-parser.service";

import { TranslationsParserTool } from "./TranslationsParserTool";

const SOURCE: string = "C:\\game\\gamedata";
const OUTPUT: string = "C:\\work\\translations";

function answer(request: TranslationsParseRequest): TranslationParseResult {
  return mockTranslationParseResult({
    isDryRun: request.isDryRun,
    language: request.language,
    census: mockTranslationParseCensus({ filesRead: 4, filesCreated: 1, entriesRead: 80 }),
  });
}

function renderTool(): RenderResult {
  return renderWithProviders(<TranslationsParserTool />, {
    route: "/translations-parser",
    bindings: [TranslationsParserService],
  });
}

function sentRequests(): Array<TranslationsParseRequest> {
  return listMockInvokeCalls(ETranslationsCommand.PARSE_PROJECT).map((args) => args.request);
}

describe("TranslationsParserTool", () => {
  beforeEach(() => {
    clearLocalStorage();
    mockIpcCommands({ [ETranslationsCommand.PARSE_PROJECT]: ({ request }) => answer(request) });
  });

  it("offers neither an import nor a preview until both the source and the output are named", async () => {
    setLocalStorageValue("xrf.form.translations-parser.source", SOURCE);

    const { findByRole, getByRole } = renderTool();
    const submit = await findByRole("button", { name: "Import" });
    const preview = getByRole("button", { name: "Preview" });

    expect(submit).toBeDisabled();
    expect(preview).toBeDisabled();

    fireEvent.change(getByRole("textbox", { name: "Output" }), { target: { value: OUTPUT } });

    await waitFor(() => expect(submit).toBeEnabled());
    expect(preview).toBeEnabled();
  });

  it("imports in English, keeping existing text, by default", async () => {
    setLocalStorageValue("xrf.form.translations-parser.source", SOURCE);
    setLocalStorageValue("xrf.form.translations-parser.destination", OUTPUT);

    const { findByRole, findByText } = renderTool();
    const submit = await findByRole("button", { name: "Import" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText("Imported 80 entries as 'eng'")).toBeInTheDocument();
    expect(sentRequests()).toEqual([
      expect.objectContaining({
        language: ETranslationLanguage.ENGLISH,
        outputDir: OUTPUT,
        isOverwrite: false,
        isDryRun: false,
      }),
    ]);
  });

  it("replaces existing text in the remembered language once asked to", async () => {
    setLocalStorageValue("xrf.form.translations-parser.source", SOURCE);
    setLocalStorageValue("xrf.form.translations-parser.destination", OUTPUT);
    setLocalStorageValue("xrf.form.translations-parser.language", ETranslationLanguage.POLISH);

    const { findByRole, findByText, getByRole } = renderTool();
    const submit = await findByRole("button", { name: "Import" });

    fireEvent.click(getByRole("switch", { name: /Replace existing text/ }));
    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText("Imported 80 entries as 'pol'")).toBeInTheDocument();
    expect(sentRequests()).toEqual([
      expect.objectContaining({ language: ETranslationLanguage.POLISH, isOverwrite: true, isDryRun: false }),
    ]);
  });

  it("previews without writing, and offers no output to show for it", async () => {
    setLocalStorageValue("xrf.form.translations-parser.source", SOURCE);
    setLocalStorageValue("xrf.form.translations-parser.destination", OUTPUT);

    const { findByRole, findByText, queryByRole } = renderTool();
    const preview = await findByRole("button", { name: "Preview" });

    await waitFor(() => expect(preview).toBeEnabled());
    fireEvent.click(preview);

    expect(await findByText("Preview only — would import 80 entries as 'eng'")).toBeInTheDocument();
    expect(sentRequests()).toEqual([expect.objectContaining({ isDryRun: true })]);
    expect(queryByRole("button", { name: "Show output" })).not.toBeInTheDocument();
  });

  it("offers the written output once an import wrote it", async () => {
    setLocalStorageValue("xrf.form.translations-parser.source", SOURCE);
    setLocalStorageValue("xrf.form.translations-parser.destination", OUTPUT);

    const { findByRole, findByText } = renderTool();
    const submit = await findByRole("button", { name: "Import" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText("Imported 80 entries as 'eng'")).toBeInTheDocument();
    expect(await findByRole("button", { name: "Show output" })).toBeInTheDocument();
  });
});
