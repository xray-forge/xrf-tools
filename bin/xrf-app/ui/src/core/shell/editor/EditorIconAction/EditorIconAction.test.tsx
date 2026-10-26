import { describe, expect, it, jest } from "@jest/globals";
import { default as SaveIcon } from "@mui/icons-material/Save";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorIconAction } from "./EditorIconAction";

describe("EditorIconAction", () => {
  it("keeps the action name, description, and identity on the button", async () => {
    const onClick = jest.fn();
    const { getByRole, getByTestId, findByRole } = renderWithProviders(
      <EditorIconAction
        data-testid={"save-action"}
        id={"save-file"}
        className={"file-action"}
        label={"Save file"}
        description={"Write the current edits to disk"}
        icon={<SaveIcon />}
        onClick={onClick}
      />
    );

    const button: HTMLElement = getByRole("button", { name: "Save file" });

    expect(getByTestId("save-action")).toBe(button);
    expect(button).toHaveAttribute("id", "save-file");
    expect(button).toHaveClass("file-action");
    expect(button).toHaveAccessibleDescription("Write the current edits to disk");

    await userEvent.hover(button);

    expect(await findByRole("tooltip")).toHaveTextContent("Write the current edits to disk");
    expect(button).toHaveAccessibleName("Save file");

    await userEvent.click(button);

    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("supports keyboard activation", async () => {
    const onClick = jest.fn();
    const { getByRole } = renderWithProviders(
      <EditorIconAction label={"Save file"} description={"Write edits"} icon={<SaveIcon />} onClick={onClick} />
    );

    await userEvent.tab();

    expect(getByRole("button", { name: "Save file" })).toHaveFocus();

    await userEvent.keyboard("{Enter}");

    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("explains a disabled action without allowing activation", async () => {
    const onClick = jest.fn();
    const { getByRole, getByTitle, findByRole } = renderWithProviders(
      <EditorIconAction
        label={"Save file"}
        description={"Nothing to save"}
        icon={<SaveIcon />}
        isDisabled={true}
        onClick={onClick}
      />
    );

    const button: HTMLElement = getByRole("button", { name: "Save file" });
    const wrapper: HTMLElement = getByTitle("Nothing to save");

    expect(button).toBeDisabled();
    expect(button).toHaveAccessibleDescription("Nothing to save");

    await userEvent.hover(wrapper);

    expect(await findByRole("tooltip")).toHaveTextContent("Nothing to save");

    await userEvent.click(wrapper);
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    expect(button).not.toHaveFocus();
    expect(onClick).not.toHaveBeenCalled();
  });
});
