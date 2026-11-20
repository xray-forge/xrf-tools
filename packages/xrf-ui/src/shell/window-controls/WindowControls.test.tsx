import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { WindowControls } from "./WindowControls";

describe("WindowControls", () => {
  it("puts minimize, maximize and close in the system's order and calls each one's action", async () => {
    const onMinimize = jest.fn();
    const onToggleMaximize = jest.fn();
    const onClose = jest.fn();
    const { getAllByRole, getByRole } = renderWithTheme(
      <WindowControls
        isMaximized={false}
        onMinimize={onMinimize}
        onToggleMaximize={onToggleMaximize}
        onClose={onClose}
      />
    );

    expect(getAllByRole("button").map((button: HTMLElement) => button.getAttribute("aria-label"))).toEqual([
      "Minimize",
      "Maximize",
      "Close",
    ]);

    await userEvent.click(getByRole("button", { name: "Minimize" }));
    await userEvent.click(getByRole("button", { name: "Maximize" }));
    await userEvent.click(getByRole("button", { name: "Close" }));

    expect(onMinimize).toHaveBeenCalledTimes(1);
    expect(onToggleMaximize).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("offers to restore a maximized window", () => {
    const { getByRole, queryByRole } = renderWithTheme(
      <WindowControls isMaximized={true} onMinimize={jest.fn()} onToggleMaximize={jest.fn()} onClose={jest.fn()} />
    );

    expect(getByRole("button", { name: "Restore down" })).toBeInTheDocument();
    expect(queryByRole("button", { name: "Maximize" })).not.toBeInTheDocument();
  });
});
