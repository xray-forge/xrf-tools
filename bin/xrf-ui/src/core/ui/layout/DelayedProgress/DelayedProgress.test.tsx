import { describe, expect, it } from "@jest/globals";

import { renderWithProviders } from "@/fixtures/utils/render";

import { DelayedProgress } from "./DelayedProgress";

describe("DelayedProgress", () => {
  it("names the pending operation while preserving the delayed reveal", () => {
    const { getByRole, getByTestId } = renderWithProviders(
      <DelayedProgress data-testid={"loading-source"} label={"Reading export source…"} />
    );

    expect(getByRole("progressbar", { hidden: true })).toHaveAttribute("aria-label", "Reading export source…");
    expect(getByRole("status", { hidden: true })).toHaveTextContent("Reading export source…");
    expect(getByTestId("loading-source")).toHaveStyle({ visibility: "hidden", animationDelay: "500ms" });
  });
});
