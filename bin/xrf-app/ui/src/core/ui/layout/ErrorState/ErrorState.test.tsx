import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { ErrorState } from "./ErrorState";

describe("ErrorState", () => {
  it("announces the failure and lets the owner retry through the keyboard", async () => {
    const onRetry = jest.fn();
    const { getByRole, getByTestId } = renderWithProviders(
      <ErrorState
        data-testid={"failed-source"}
        id={"source-error"}
        className={"source-state"}
        title={"Could not read this source"}
        description={"The source file is unavailable."}
        onRetry={onRetry}
      />
    );
    const alert = getByRole("alert");

    expect(alert).toBe(getByTestId("failed-source"));
    expect(alert).toHaveAttribute("id", "source-error");
    expect(alert).toHaveClass("source-state");
    expect(alert).toHaveTextContent("Could not read this source");
    expect(alert).toHaveTextContent("The source file is unavailable.");

    await userEvent.tab();

    expect(getByRole("button", { name: "Retry" })).toHaveFocus();

    await userEvent.keyboard("{Enter}");

    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("offers no retry when its owner has no request to repeat", () => {
    const { getByRole, queryByRole } = renderWithProviders(
      <ErrorState title={"Could not show this encoding"} description={"Unsupported format"} />
    );

    expect(getByRole("alert")).toHaveTextContent("Unsupported format");
    expect(queryByRole("button")).not.toBeInTheDocument();
  });
});
