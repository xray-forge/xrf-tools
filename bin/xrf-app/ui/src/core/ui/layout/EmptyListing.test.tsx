import { describe, expect, it } from "@jest/globals";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EmptyListing } from "./EmptyListing";

describe("EmptyListing", () => {
  it("says what is absent, and carries the identity its caller gave it", () => {
    const { getByTestId, getByText } = renderWithProviders(
      <EmptyListing data-testid={"no-configs"} id={"configs-empty"} className={"configs-state"} label={"No configs."} />
    );
    const listing = getByTestId("no-configs");

    expect(listing).toHaveAttribute("id", "configs-empty");
    expect(listing).toHaveClass("configs-state");
    expect(getByText("No configs.")).toBeInTheDocument();
  });

  it("names itself when the caller does not, so a listing is findable without one", () => {
    const { getByTestId } = renderWithProviders(<EmptyListing label={"Nothing here."} />);

    expect(getByTestId("empty-listing")).toBeInTheDocument();
  });

  it("lets a caller's class win, which is the whole reason it takes one", () => {
    // `cn` is merge-backed, so a caller that wants different padding gets it rather than getting both.
    const { getByTestId } = renderWithProviders(<EmptyListing data-testid={"tight"} className={"p-0"} label={"—"} />);

    expect(getByTestId("tight")).toHaveClass("p-0");
    expect(getByTestId("tight")).not.toHaveClass("p-4");
  });
});
