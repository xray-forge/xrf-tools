import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { KeyboardEvent } from "react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorFilterInput } from "./EditorFilterInput";

describe("EditorFilterInput", () => {
  it("labels the controlled input and requests query edits", async () => {
    const onQueryChange = jest.fn();
    const { getByRole, getByTestId } = renderWithProviders(
      <EditorFilterInput
        data-testid={"motion-filter"}
        id={"motions-query"}
        className={"motion-control"}
        query={"walk"}
        placeholder={"Filter motions"}
        ariaLabel={"Filter available motions"}
        onQueryChange={onQueryChange}
      />
    );
    const input = getByRole("textbox", { name: "Filter available motions" });

    expect(input).toHaveAttribute("id", "motions-query");
    expect(input).toHaveAttribute("placeholder", "Filter motions");
    expect(getByTestId("motion-filter")).toHaveClass("motion-control");

    await userEvent.type(input, "s");

    expect(onQueryChange).toHaveBeenCalledWith("walks");
    expect(input).toHaveValue("walk");
  });

  it("requests an empty query and keeps focus when the clear button disappears", async () => {
    const onQueryChange = jest.fn();
    const { getByRole, queryByRole, rerender } = renderWithProviders(
      <EditorFilterInput
        query={"walk"}
        placeholder={"Filter motions"}
        ariaLabel={"Motions"}
        onQueryChange={onQueryChange}
      />
    );
    const input = getByRole("textbox", { name: "Motions" });

    await userEvent.click(getByRole("button", { name: "Clear filter" }));

    expect(onQueryChange).toHaveBeenCalledTimes(1);
    expect(onQueryChange).toHaveBeenCalledWith("");
    expect(input).toHaveFocus();
    expect(input).toHaveValue("walk");

    rerender(
      <>
        <EditorFilterInput
          query={""}
          placeholder={"Filter motions"}
          ariaLabel={"Motions"}
          onQueryChange={onQueryChange}
        />
      </>
    );

    expect(queryByRole("button", { name: "Clear filter" })).not.toBeInTheDocument();
    expect(input).toHaveFocus();
    expect(input).toHaveValue("");
  });

  it("forwards input navigation and lets keyboard clearing use the owner's cleanup", async () => {
    const onClear = jest.fn();
    const onQueryChange = jest.fn();
    const onKeyDown = jest.fn((event: KeyboardEvent<HTMLElement>) => {
      if (event.key === "ArrowDown" || event.key === "Enter") {
        event.preventDefault();
      }
    });
    const { getByRole } = renderWithProviders(
      <EditorFilterInput
        query={"walk"}
        placeholder={"Filter motions"}
        ariaLabel={"Motions"}
        onClear={onClear}
        onQueryChange={onQueryChange}
        onKeyDown={onKeyDown}
      />
    );
    const input = getByRole("textbox", { name: "Motions" });

    expect(fireEvent.keyDown(input, { key: "ArrowDown" })).toBe(false);
    expect(fireEvent.keyDown(input, { key: "Enter" })).toBe(false);
    expect(onKeyDown).toHaveBeenCalledWith(expect.objectContaining({ key: "ArrowDown", target: input }));
    expect(onKeyDown).toHaveBeenCalledWith(expect.objectContaining({ key: "Enter", target: input }));

    await userEvent.tab();
    await userEvent.tab();

    expect(getByRole("button", { name: "Clear filter" })).toHaveFocus();
    onKeyDown.mockClear();

    await userEvent.keyboard("{Enter}");

    expect(onClear).toHaveBeenCalledTimes(1);
    expect(onQueryChange).not.toHaveBeenCalled();
    expect(onKeyDown).not.toHaveBeenCalled();
    expect(input).toHaveFocus();
  });
});
