import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { VisualPreviewEmpty } from "@/core/visuals/components/preview/VisualPreviewEmpty";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("VisualPreviewEmpty", () => {
  it("says nothing is open, with nothing to retry", () => {
    const render: RenderResult = renderWithProviders(<VisualPreviewEmpty onRetry={jest.fn()} />);

    expect(render.getByText("No visual open")).toBeInTheDocument();
    expect(render.queryByRole("button", { name: "Retry" })).toBeNull();
  });

  it("states a failed open and offers the retry beside it", () => {
    const onRetry = jest.fn();
    const render: RenderResult = renderWithProviders(
      <VisualPreviewEmpty error={"not an ogf file"} onRetry={onRetry} />
    );

    expect(render.getByText("Could not open this visual")).toBeInTheDocument();
    expect(render.getByText("not an ogf file")).toBeInTheDocument();

    fireEvent.click(render.getByRole("button", { name: "Retry" }));

    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("offers no retry to a surface that cannot repeat its open", () => {
    const render: RenderResult = renderWithProviders(<VisualPreviewEmpty error={"not an ogf file"} />);

    expect(render.getByText("Could not open this visual")).toBeInTheDocument();
    expect(render.queryByRole("button", { name: "Retry" })).toBeNull();
  });
});
