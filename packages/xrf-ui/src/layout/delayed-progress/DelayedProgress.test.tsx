import { describe, expect, it } from "@jest/globals";

import { renderWithTheme } from "#/testing/render";

import { DelayedProgress } from "./DelayedProgress";

describe("DelayedProgress", () => {
  it("names the pending operation while preserving the delayed reveal", () => {
    const { getByRole, getByTestId } = renderWithTheme(
      <DelayedProgress data-testid={"loading-source"} label={"Reading export source…"} />
    );

    expect(getByRole("progressbar", { hidden: true })).toHaveAttribute("aria-label", "Reading export source…");
    expect(getByRole("status", { hidden: true })).toHaveTextContent("Reading export source…");
    expect(getByTestId("loading-source")).toHaveClass("invisible", "animate-delayed-reveal");
  });
});
