import { ReactElement } from "react";

import { IErrorBoundaryFallbackProps } from "./ErrorBoundary";

/**
 * Last-resort recovery UI that stays independent of the application provider stack.
 * todo: Render header controls to close the app and move the window.
 */
export function RootCrash({ error, onRetry }: IErrorBoundaryFallbackProps): ReactElement {
  return (
    <main
      style={{
        alignItems: "center",
        boxSizing: "border-box",
        color: "CanvasText",
        display: "flex",
        fontFamily: "'Segoe UI Variable Text', 'Segoe UI', 'Roboto', system-ui, sans-serif",
        justifyContent: "center",
        minHeight: "100vh",
        padding: 24,
      }}
    >
      <section
        aria-labelledby={"root-crash-title"}
        style={{
          maxWidth: 560,
          width: "100%",
        }}
      >
        <h1
          id={"root-crash-title"}
          style={{
            fontSize: 20,
            margin: "0 0 8px",
          }}
        >
          Something went wrong
        </h1>

        <p
          style={{
            lineHeight: 1.5,
            margin: 0,
          }}
        >
          XRF tools stopped rendering. Try again, or reload the window to start fresh.
        </p>

        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: 8,
            marginTop: 24,
          }}
        >
          <button
            style={{
              cursor: "pointer",
              font: "inherit",
              padding: "6px 12px",
            }}
            type={"button"}
            onClick={onRetry}
          >
            Try again
          </button>

          <button
            style={{
              cursor: "pointer",
              font: "inherit",
              padding: "6px 12px",
            }}
            type={"button"}
            onClick={() => window.location.reload()}
          >
            Reload window
          </button>
        </div>

        <details
          style={{
            marginTop: 24,
          }}
        >
          <summary>Details</summary>
          <pre
            style={{
              fontFamily: "'Cascadia Mono', 'Consolas', monospace",
              fontSize: 12,
              overflowWrap: "anywhere",
              whiteSpace: "pre-wrap",
            }}
          >
            {error.stack ?? String(error)}
          </pre>
        </details>
      </section>
    </main>
  );
}
