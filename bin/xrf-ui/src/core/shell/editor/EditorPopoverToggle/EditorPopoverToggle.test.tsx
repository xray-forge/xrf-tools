import { describe, expect, it, jest } from "@jest/globals";
import { default as FoggyIcon } from "@mui/icons-material/Foggy";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorPopoverToggle } from "./EditorPopoverToggle";

function renderToggle(isOn: boolean, onToggle: () => void = () => {}) {
  return renderWithProviders(
    <EditorPopoverToggle
      label={"Fog"}
      description={"Fog total at 350 m"}
      icon={<FoggyIcon />}
      isOn={isOn}
      toggleLabel={"Draw the fog"}
      onToggle={onToggle}
    >
      <button>Distance</button>
    </EditorPopoverToggle>
  );
}

describe("EditorPopoverToggle", () => {
  it("opens its settings on a click, led by the toggle as a checkbox", async () => {
    const onToggle = jest.fn();
    const { getByRole, findByRole } = renderToggle(true, onToggle);

    await userEvent.click(getByRole("button", { name: "Fog" }));

    const dialog: HTMLElement = await findByRole("dialog", { name: "Fog" });
    const checkbox: HTMLElement = getByRole("checkbox", { name: "Draw the fog" });

    expect(dialog).toContainElement(getByRole("button", { name: "Distance" }));
    expect(checkbox).toBeChecked();
    expect(onToggle).not.toHaveBeenCalled();

    await userEvent.click(checkbox);

    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("turns over on a right click without opening, and keeps the browser's menu away", () => {
    const onToggle = jest.fn();
    const { getByRole, queryByRole } = renderToggle(false, onToggle);

    const isDefaultAllowed: boolean = fireEvent.contextMenu(getByRole("button", { name: "Fog" }));

    expect(onToggle).toHaveBeenCalledTimes(1);
    expect(isDefaultAllowed).toBe(false);
    expect(queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("says what its settings are at and what a right click does", () => {
    expect(renderToggle(true).getByRole("button", { name: "Fog" })).toHaveAccessibleDescription(
      "Fog total at 350 m. Right-click to turn off"
    );
  });
});
