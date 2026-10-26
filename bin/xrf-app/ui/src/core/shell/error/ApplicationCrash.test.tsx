import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ApplicationCrash } from "@/core/shell/error/ApplicationCrash";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ApplicationCrash", () => {
  it("presents shell recovery actions and the original error", () => {
    const { getByText } = renderWithProviders(
      <ApplicationCrash error={new Error("render exploded")} onRetry={jest.fn()} />
    );

    expect(getByText("This tool stopped rendering")).toBeInTheDocument();
    expect(getByText("Try again")).toBeInTheDocument();
    expect(getByText("Go home")).toBeInTheDocument();
    expect(getByText("Reload window")).toBeInTheDocument();
    expect(getByText(/render exploded/)).toBeInTheDocument();
  });

  it("delegates retry to the error boundary", async () => {
    const onRetry = jest.fn();
    const { getByText } = renderWithProviders(<ApplicationCrash error={new Error("failed")} onRetry={onRetry} />);

    await userEvent.click(getByText("Try again"));

    expect(onRetry).toHaveBeenCalledTimes(1);
  });
});
