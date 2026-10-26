import { describe, expect, it, jest } from "@jest/globals";
import { listItemButtonClasses } from "@mui/material";
import { userEvent } from "@testing-library/user-event";

import { EditorSideMenu } from "@/core/shell/editor/EditorSideMenu";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("EditorSideMenu", () => {
  it("lists sections and actions together", () => {
    const { getByText } = renderWithProviders(
      <EditorSideMenu sections={[{ label: "Header" }, { label: "Alife" }]} actions={[{ label: "Save" }]} />
    );

    expect(getByText("Header")).toBeInTheDocument();
    expect(getByText("Alife")).toBeInTheDocument();
    expect(getByText("Save")).toBeInTheDocument();
  });

  it("invokes the item that was clicked", async () => {
    const onHeader = jest.fn();
    const onAlife = jest.fn();

    const { getByText } = renderWithProviders(
      <EditorSideMenu
        sections={[
          { label: "Header", onClick: onHeader },
          { label: "Alife", onClick: onAlife },
        ]}
      />
    );

    await userEvent.click(getByText("Alife"));

    expect(onAlife).toHaveBeenCalledTimes(1);
    expect(onHeader).not.toHaveBeenCalled();
  });

  it("marks the selected section so the menu shows where you are", () => {
    const { getByText } = renderWithProviders(
      <EditorSideMenu sections={[{ label: "Header", isSelected: true }, { label: "Alife" }]} />
    );

    expect(getByText("Header").closest("[role='button']")).toHaveClass(listItemButtonClasses.selected);
    expect(getByText("Alife").closest("[role='button']")).not.toHaveClass(listItemButtonClasses.selected);
  });

  it("makes a disabled action unclickable rather than merely ignoring the click", () => {
    const { getByText } = renderWithProviders(
      <EditorSideMenu actions={[{ label: "Save", isDisabled: true, onClick: jest.fn() }]} />
    );

    const action: Element = getByText("Save").closest("[role='button']")!;

    // Asserted rather than clicked on purpose: the element carries `pointer-events: none`, so
    // `userEvent` refuses to click it at all - which is the behaviour being checked.
    expect(action).toHaveClass(listItemButtonClasses.disabled);
    expect(action).toHaveAttribute("aria-disabled", "true");
  });

  it("renders arbitrary content in place of a section list", () => {
    const { getByText, queryByRole } = renderWithProviders(
      <EditorSideMenu>
        <div>tree goes here</div>
      </EditorSideMenu>
    );

    expect(getByText("tree goes here")).toBeInTheDocument();
    expect(queryByRole("button")).not.toBeInTheDocument();
  });

  it("keeps the header and the actions outside the scrolling middle", () => {
    const { getByText } = renderWithProviders(
      <EditorSideMenu header={<div>search</div>} sections={[{ label: "Header" }]} actions={[{ label: "Save" }]} />
    );

    const scrolling: Element = getByText("Header").closest("ul")!.parentElement!;

    expect(getComputedStyle(scrolling).overflowY).toBe("auto");
    expect(scrolling).not.toContainElement(getByText("search"));
    expect(scrolling).not.toContainElement(getByText("Save"));
  });
});
