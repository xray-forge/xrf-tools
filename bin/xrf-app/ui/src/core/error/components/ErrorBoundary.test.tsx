import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { ErrorBoundary, IErrorBoundaryFallbackProps } from "./ErrorBoundary";

function Boom(): ReactElement {
  throw new Error("render exploded");
}

function RenderFallback({ error, onRetry }: IErrorBoundaryFallbackProps): ReactElement {
  return (
    <>
      <span>Fallback: {error.message}</span>
      <button onClick={onRetry}>Try again</button>
    </>
  );
}

/**
 * Runs a render while silencing React's expected error log.
 *
 * @param run - Render operation that triggers React's expected error log.
 * @returns Result returned by the render operation.
 */
function withSilencedConsole(run: () => RenderResult): RenderResult {
  const spy = jest.spyOn(console, "error").mockImplementation(() => {});

  try {
    return run();
  } finally {
    spy.mockRestore();
  }
}

describe("ErrorBoundary", () => {
  it("reports a caught crash, which is the only place a swallowed one is observable", () => {
    const onCaught = jest.fn();

    withSilencedConsole(() =>
      renderWithProviders(
        <ErrorBoundary fallback={RenderFallback} onCaught={onCaught}>
          <Boom />
        </ErrorBoundary>
      )
    );

    // React does not rethrow what a boundary caught, so no global handler ever sees this.
    expect(onCaught).toHaveBeenCalledTimes(1);

    const [error, componentStack] = onCaught.mock.calls[0] as [Error, string];

    expect(error.message).toBe("render exploded");
    expect(componentStack).toContain("Boom");
  });

  it("shows the fallback instead of taking the window down", () => {
    const { getByText } = withSilencedConsole(() =>
      renderWithProviders(
        <ErrorBoundary fallback={RenderFallback}>
          <Boom />
        </ErrorBoundary>
      )
    );

    expect(getByText("Fallback: render exploded")).toBeInTheDocument();
  });

  it("renders children untouched when nothing throws", () => {
    const { getByText, queryByText } = renderWithProviders(
      <ErrorBoundary fallback={RenderFallback}>
        <div>healthy</div>
      </ErrorBoundary>
    );

    expect(getByText("healthy")).toBeInTheDocument();
    expect(queryByText("Fallback: render exploded")).not.toBeInTheDocument();
  });

  it("re-renders the subtree on retry rather than reloading", async () => {
    function Flaky(): ReactElement {
      if (!(globalThis as { isFixed?: boolean }).isFixed) {
        throw new Error("not yet");
      }

      return <div>recovered</div>;
    }

    const { getByText } = withSilencedConsole(() =>
      renderWithProviders(
        <ErrorBoundary fallback={RenderFallback}>
          <Flaky />
        </ErrorBoundary>
      )
    );

    (globalThis as { isFixed?: boolean }).isFixed = true;

    await userEvent.click(getByText("Try again"));

    expect(getByText("recovered")).toBeInTheDocument();

    delete (globalThis as { isFixed?: boolean }).isFixed;
  });

  it("clears the failure when the reset key changes", async () => {
    function Host(): ReactElement {
      const [route, setRoute] = useState<string>("/broken");

      return (
        <>
          <button onClick={() => setRoute("/healthy")}>navigate</button>

          <ErrorBoundary resetKey={route} fallback={RenderFallback}>
            {route === "/broken" ? <Boom /> : <div>next tool</div>}
          </ErrorBoundary>
        </>
      );
    }

    const { getByText, queryByText } = withSilencedConsole(() => renderWithProviders(<Host />));

    expect(getByText("Fallback: render exploded")).toBeInTheDocument();

    // Without the reset the fallback outlives the route that caused it, and the next tool never
    // renders - the crash follows the user around the application.
    await userEvent.click(getByText("navigate"));

    expect(getByText("next tool")).toBeInTheDocument();
    expect(queryByText("Fallback: render exploded")).not.toBeInTheDocument();
  });
});
