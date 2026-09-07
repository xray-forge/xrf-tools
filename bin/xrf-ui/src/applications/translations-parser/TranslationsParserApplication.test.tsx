import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { TranslationsParserService } from "@/applications/translations-parser/services/parser";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TranslationsParserApplication } from "./TranslationsParserApplication";

describe("TranslationsParserApplication", () => {
  beforeEach(() => window.localStorage.clear());

  it("remembers the selected language but starts with replacement off on each mount", async () => {
    window.localStorage.setItem("xrf.form.translations-parser.language", "ukr");

    const { container } = mockInjectedService(TranslationsParserService);
    const view = renderWithProviders(<TranslationsParserApplication />, { container, route: "/translations-parser" });
    const overwrite = view.getByRole("switch", { name: "Replace existing text Optional" });

    expect(view.getByRole("combobox", { name: /^Language/ })).toHaveTextContent("ukr");
    expect(overwrite).not.toBeChecked();
    expect(overwrite).toHaveAccessibleDescription("Text already in the output is kept unless this is on");

    await userEvent.click(view.getByText("Replace existing text"));

    expect(overwrite).toBeChecked();

    view.unmount();

    const next = renderWithProviders(<TranslationsParserApplication />, { container, route: "/translations-parser" });

    expect(next.getByRole("combobox", { name: /^Language/ })).toHaveTextContent("ukr");
    expect(next.getByRole("switch", { name: "Replace existing text Optional" })).not.toBeChecked();
  });
});
