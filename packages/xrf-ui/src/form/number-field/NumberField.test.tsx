import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { renderWithTheme } from "#/testing/render";

import { NumberField } from "./NumberField";

function toTenths(value: number): string {
  return String(Math.round(value * 10) / 10);
}

describe("NumberField", () => {
  it("commits each number typed in whole and holds what is not one yet", () => {
    const onCommit = jest.fn();
    const { getByRole } = renderWithTheme(<NumberField label={"Top"} value={10} onCommit={onCommit} />);
    const input: HTMLElement = getByRole("spinbutton", { name: "Top" });

    fireEvent.change(input, { target: { value: "" } });

    expect(onCommit).not.toHaveBeenCalled();
    expect(input).toHaveAttribute("aria-invalid", "true");

    fireEvent.change(input, { target: { value: "-4.5" } });

    expect(onCommit).toHaveBeenCalledWith(-4.5);
    expect(input).toHaveAttribute("aria-invalid", "false");
  });

  it("takes a value set from outside, spelled as the field formats it", () => {
    const { getByRole, rerender } = renderWithTheme(
      <NumberField label={"Least x"} value={1} format={toTenths} onCommit={jest.fn()} />
    );

    rerender(<NumberField label={"Least x"} value={12.345} format={toTenths} onCommit={jest.fn()} />);

    expect(getByRole("spinbutton", { name: "Least x" })).toHaveValue(12.3);
  });

  it("keeps the spelling of the number just typed when the value comes back as it", () => {
    const onCommit = jest.fn();
    const { getByRole, rerender } = renderWithTheme(<NumberField label={"Top"} value={1} onCommit={onCommit} />);
    const input: HTMLInputElement = getByRole("spinbutton", { name: "Top" }) as HTMLInputElement;

    fireEvent.change(input, { target: { value: "2.50" } });
    rerender(<NumberField label={"Top"} value={2.5} onCommit={onCommit} />);

    expect(input.value).toBe("2.50");
  });
});
