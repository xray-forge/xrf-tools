import { expect, it, jest } from "@jest/globals";
import { render } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { ErrorBoundary } from "./ErrorBoundary";
import { RootCrash } from "./RootCrash";

function FailingComponent(): ReactElement {
  throw new Error("root exploded");
}

it("renders root recovery without application providers", async () => {
  const error: Error = new Error("provider exploded");
  const onRetry = jest.fn();

  const { getByRole, getByText } = render(<RootCrash error={error} onRetry={onRetry} />);

  expect(getByRole("heading", { name: "Something went wrong" })).toBeInTheDocument();
  expect(getByText(/provider exploded/)).toBeInTheDocument();

  await userEvent.click(getByRole("button", { name: "Try again" }));

  expect(onRetry).toHaveBeenCalledTimes(1);
});

it("recovers when the root subtree crashes", () => {
  const consoleError = jest.spyOn(console, "error").mockImplementation(() => {});

  try {
    const { getByRole } = render(
      <ErrorBoundary fallback={RootCrash}>
        <FailingComponent />
      </ErrorBoundary>
    );

    expect(getByRole("heading", { name: "Something went wrong" })).toBeInTheDocument();
  } finally {
    consoleError.mockRestore();
  }
});
