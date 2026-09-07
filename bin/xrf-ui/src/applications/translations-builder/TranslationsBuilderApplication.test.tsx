import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { TranslationsBuilderService } from "@/applications/translations-builder/services/builder";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TranslationsBuilderApplication } from "./TranslationsBuilderApplication";

describe("TranslationsBuilderApplication", () => {
  beforeEach(() => window.localStorage.clear());

  it("starts with all languages and lets the sort label toggle the explained input", async () => {
    const { container } = mockInjectedService(TranslationsBuilderService);
    const { getByRole, getByText } = renderWithProviders(<TranslationsBuilderApplication />, {
      container,
      route: "/translations-builder",
    });
    const sort = getByRole("switch", { name: "Sort ids Optional" });

    expect(getByRole("combobox", { name: /^Language/ })).toHaveTextContent("all");
    expect(sort).toBeChecked();
    expect(sort).toHaveAccessibleDescription("Off preserves the order each source declares them in");

    await userEvent.click(getByText("Sort ids"));

    expect(sort).not.toBeChecked();
  });
});
