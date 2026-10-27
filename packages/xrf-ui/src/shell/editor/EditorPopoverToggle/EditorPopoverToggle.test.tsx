import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { EditorPopoverToggle } from "./EditorPopoverToggle";

import { FoggyIcon } from "#/icons";
import { renderWithTheme } from "#/testing/render";

function renderToggle(isOn: boolean, onToggle: () => void = () => {}, isDisabled: boolean = false) {
  return renderWithTheme(
    <EditorPopoverToggle
      label={"Fog"}
      description={"Fog total at 350 m"}
      icon={<FoggyIcon />}
      isOn={isOn}
      isDisabled={isDisabled}
      toggleLabel={"Draw the fog"}
      onToggle={onToggle}
    >
      <button>Distance</button>
    </EditorPopoverToggle>
  );
}

describe("EditorPopoverToggle", () => {
  it("opens its settings on a right click, led by the toggle as a checkbox, and keeps the browser's menu away", async () => {
    const onToggle = jest.fn();
    const { getByRole, findByRole } = renderToggle(true, onToggle);

    const isDefaultAllowed: boolean = fireEvent.contextMenu(getByRole("button", { name: "Fog" }));

    const dialog: HTMLElement = await findByRole("dialog", { name: "Fog" });
    const checkbox: HTMLElement = getByRole("checkbox", { name: "Draw the fog" });

    expect(isDefaultAllowed).toBe(false);
    expect(dialog).toContainElement(getByRole("button", { name: "Distance" }));
    expect(checkbox).toBeChecked();
    expect(onToggle).not.toHaveBeenCalled();

    await userEvent.click(checkbox);

    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("turns over on a click without opening", async () => {
    const onToggle = jest.fn();
    const { getByRole, queryByRole } = renderToggle(false, onToggle);

    await userEvent.click(getByRole("button", { name: "Fog" }));

    expect(onToggle).toHaveBeenCalledTimes(1);
    expect(queryByRole("dialog")).not.toBeInTheDocument();
  });

  // The corner cue tells a toggle from a trigger a click opens, as long as a click turns it over.
  it("cues a click that turns it over, and one that opens it while it cannot", () => {
    const active = renderToggle(true);

    expect(active.getByRole("button", { name: "Fog" })).toHaveAttribute("data-cue", "toggle");
    active.unmount();

    expect(renderToggle(true, () => {}, true).getByRole("button", { name: "Fog" })).toHaveAttribute(
      "data-cue",
      "popover"
    );
  });

  it("says what its settings are at and what a right click does", () => {
    expect(renderToggle(true).getByRole("button", { name: "Fog" })).toHaveAccessibleDescription(
      "Fog total at 350 m. Right-click for its settings"
    );
  });

  // A click opens it then, so the hint would name the wrong button.
  it("offers no right click while it cannot turn over", () => {
    expect(renderToggle(true, () => {}, true).getByRole("button", { name: "Fog" })).toHaveAccessibleDescription(
      "Fog total at 350 m"
    );
  });

  it("opens on a click while it cannot turn over, saying why in place of its settings", async () => {
    const onToggle = jest.fn();
    const { getByRole, findByRole, queryByRole } = renderToggle(true, onToggle, true);

    await userEvent.click(getByRole("button", { name: "Fog" }));

    const dialog: HTMLElement = await findByRole("dialog", { name: "Fog" });

    expect(dialog).toHaveTextContent("Fog total at 350 m");
    expect(queryByRole("button", { name: "Distance" })).not.toBeInTheDocument();
    expect(getByRole("checkbox", { name: "Draw the fog" })).toBeDisabled();
    expect(onToggle).not.toHaveBeenCalled();
  });

  it("says why its settings do nothing in their place, and still turns over from its checkbox", async () => {
    const onToggle = jest.fn();
    const { getByRole, findByRole, queryByRole } = renderWithTheme(
      <EditorPopoverToggle
        label={"Fog"}
        description={"Fog as the weather sets it"}
        icon={<FoggyIcon />}
        isOn
        toggleLabel={"Draw the fog"}
        lockedReason={"The weather sets it"}
        onToggle={onToggle}
      >
        <button>Distance</button>
      </EditorPopoverToggle>
    );

    fireEvent.contextMenu(getByRole("button", { name: "Fog" }));

    const dialog: HTMLElement = await findByRole("dialog", { name: "Fog" });

    expect(dialog).toHaveTextContent("The weather sets it");
    expect(queryByRole("button", { name: "Distance" })).not.toBeInTheDocument();

    await userEvent.click(getByRole("checkbox", { name: "Draw the fog" }));

    expect(onToggle).toHaveBeenCalledTimes(1);
  });
});
