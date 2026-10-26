import { describe, expect, it } from "@jest/globals";

import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ApplicationLoader", () => {
  it("holds the progress indicator back rather than drawing it at once", () => {
    const { getByRole, getByTestId } = renderWithProviders(<ApplicationLoader data-testid={"loader"} />);

    expect(getByTestId("loader")).toHaveClass("invisible", "animate-delayed-reveal");
    expect(getByRole("progressbar", { hidden: true })).toBeInTheDocument();
  });
});
