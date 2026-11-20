import { describe, expect, it } from "@jest/globals";

import { renderWithTheme } from "#/testing/render";

import { StatusPill } from "./StatusPill";

describe("StatusPill", () => {
  it("names the status inline, outlined, and offers nothing to press", () => {
    const { getByTestId, queryByRole } = renderWithTheme(<StatusPill label={"Planned"} />);
    const pill: HTMLElement = getByTestId("status-pill");

    expect(pill.tagName.toLowerCase()).toBe("span");
    expect(pill).toHaveTextContent("Planned");
    expect(pill).toHaveClass("border");
    expect(queryByRole("button")).not.toBeInTheDocument();
  });
});
