import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { TranslationLanguageField } from "./TranslationLanguageField";

describe("TranslationLanguageField", () => {
  it("labels the controlled selection and requests a language change", async () => {
    const onChange = jest.fn();
    const { getByRole, getByTestId, getByText, rerender } = renderWithProviders(
      <TranslationLanguageField
        data-testid={"language"}
        id={"build-language"}
        className={"build-field"}
        description={"Languages to compile"}
        value={"all"}
        isAllAllowed
        onChange={onChange}
      />
    );
    const select = getByRole("combobox", { name: /^Language/ });

    expect(select).toHaveAttribute("id", "build-language");
    expect(select).toHaveAccessibleDescription("Languages to compile");
    expect(getByTestId("language")).toHaveClass("build-field");

    await userEvent.click(getByText("Language"));

    expect(select).toHaveFocus();

    await userEvent.keyboard("{ArrowDown}");

    expect(getByRole("option", { name: "all" })).toBeInTheDocument();

    await userEvent.click(getByRole("option", { name: "ukr" }));

    expect(onChange).toHaveBeenCalledWith("ukr");
    expect(select).toHaveTextContent("all");

    rerender(
      <>
        <TranslationLanguageField
          id={"build-language"}
          description={"Languages to compile"}
          value={"ukr"}
          isAllAllowed
          onChange={onChange}
        />
      </>
    );

    expect(select).toHaveTextContent("ukr");
  });

  it("offers only individual languages unless all is allowed", async () => {
    const { getByRole, queryByRole } = renderWithProviders(
      <TranslationLanguageField description={"Language to import"} value={"eng"} onChange={jest.fn()} />
    );

    await userEvent.click(getByRole("combobox", { name: /^Language/ }));

    expect(getByRole("option", { name: "eng", selected: true })).toBeInTheDocument();
    expect(getByRole("option", { name: "ukr" })).toBeInTheDocument();
    expect(queryByRole("option", { name: "all" })).not.toBeInTheDocument();
  });

  it("keeps separate labels and descriptions while a field is disabled", async () => {
    const onChange = jest.fn();
    const { getAllByRole, queryByRole } = renderWithProviders(
      <>
        <TranslationLanguageField description={"Language to build"} value={"eng"} onChange={onChange} />
        <TranslationLanguageField description={"Language to check"} value={"ukr"} isDisabled onChange={onChange} />
      </>
    );
    const [build, verify] = getAllByRole("combobox", { name: /^Language/ });

    expect(build.id).not.toBe(verify.id);
    expect(build).toHaveAccessibleDescription("Language to build");
    expect(verify).toHaveAccessibleDescription("Language to check");
    expect(verify).toHaveAttribute("aria-disabled", "true");

    await userEvent.click(verify);

    expect(queryByRole("listbox")).not.toBeInTheDocument();
    expect(onChange).not.toHaveBeenCalled();
  });
});
