import { describe, expect, it } from "@jest/globals";
import { default as TuneIcon } from "@mui/icons-material/Tune";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorPopoverAction } from "./EditorPopoverAction";

describe("EditorPopoverAction", () => {
  it("opens a named dialog from the keyboard and restores focus when dismissed", async () => {
    const { getByRole, findByRole, queryByRole, getByTestId } = renderWithProviders(
      <EditorPopoverAction
        data-testid={"options"}
        id={"preview-options"}
        className={"preview-control"}
        label={"Options"}
        description={"Adjust the preview"}
        icon={<TuneIcon />}
      >
        <button>Setting</button>
      </EditorPopoverAction>
    );
    const button: HTMLElement = getByRole("button", { name: "Options" });

    expect(getByTestId("options")).toBe(button);
    expect(button).toHaveAttribute("id", "preview-options");
    expect(button).toHaveClass("preview-control");
    expect(button).toHaveAccessibleDescription("Adjust the preview");
    expect(button).toHaveAttribute("aria-expanded", "false");

    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    const dialog: HTMLElement = await findByRole("dialog", { name: "Options" });

    expect(button).toHaveAttribute("aria-expanded", "true");
    expect(button).toHaveAttribute("aria-controls", dialog.id);

    await userEvent.keyboard("{Escape}");

    await waitFor(() => expect(queryByRole("dialog")).not.toBeInTheDocument());
    expect(button).toHaveFocus();
    expect(button).toHaveAttribute("aria-expanded", "false");
    expect(button).not.toHaveAttribute("aria-controls");
  });

  it("explains a disabled action without opening its dialog", async () => {
    const { getByRole, getByTitle, findByRole, queryByRole } = renderWithProviders(
      <EditorPopoverAction label={"Options"} description={"No preview available"} icon={<TuneIcon />} isDisabled={true}>
        <button>Setting</button>
      </EditorPopoverAction>
    );
    const wrapper: HTMLElement = getByTitle("No preview available");

    expect(getByRole("button", { name: "Options" })).toBeDisabled();

    await userEvent.hover(wrapper);

    expect(await findByRole("tooltip")).toHaveTextContent("No preview available");

    await userEvent.click(wrapper);

    expect(queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("closes when disabled and stays closed when enabled again", async () => {
    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(
      <EditorPopoverAction label={"Options"} description={"Adjust the preview"} icon={<TuneIcon />}>
        <button>Setting</button>
      </EditorPopoverAction>
    );

    await userEvent.click(getByRole("button", { name: "Options" }));

    expect(await findByRole("dialog", { name: "Options" })).toBeInTheDocument();

    rerender(
      <>
        <EditorPopoverAction
          label={"Options"}
          description={"No preview available"}
          icon={<TuneIcon />}
          isDisabled={true}
        >
          <button>Setting</button>
        </EditorPopoverAction>
      </>
    );

    await waitFor(() => expect(queryByRole("dialog")).not.toBeInTheDocument());

    rerender(
      <>
        <EditorPopoverAction label={"Options"} description={"Adjust the preview"} icon={<TuneIcon />}>
          <button>Setting</button>
        </EditorPopoverAction>
      </>
    );

    expect(getByRole("button", { name: "Options" })).toHaveAttribute("aria-expanded", "false");
    expect(queryByRole("dialog")).not.toBeInTheDocument();
  });
});
