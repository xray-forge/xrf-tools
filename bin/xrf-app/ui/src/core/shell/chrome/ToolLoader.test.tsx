import { describe, expect, it } from "@jest/globals";

import { ToolLoader } from "@/core/shell/chrome/ToolLoader";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ToolLoader", () => {
  it("holds the progress indicator back rather than drawing it at once", () => {
    const { getByRole, getByTestId } = renderWithProviders(<ToolLoader data-testid={"loader"} />);

    expect(getByTestId("loader")).toHaveClass("invisible", "animate-delayed-reveal");
    expect(getByRole("progressbar", { hidden: true })).toBeInTheDocument();
  });
});
