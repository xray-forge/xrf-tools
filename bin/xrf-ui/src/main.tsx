import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { ApplicationProvider } from "@/ApplicationProvider";
import { ApplicationRouter } from "@/ApplicationRouter";
import { ErrorBoundary } from "@/core/error/components/ErrorBoundary";
import { RootCrash } from "@/core/error/components/RootCrash";
import { suppressNativeContextMenu } from "@/lib/dom/event";
import { Logger } from "@/lib/logging";

Logger.info("Start application");

// Set once for the lifetime of the webview rather than from a component: it is host chrome policy,
// not application state, and nothing ever restores the default menu.
suppressNativeContextMenu();

createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <ErrorBoundary fallback={RootCrash}>
      <ApplicationProvider>
        <ApplicationRouter />
      </ApplicationProvider>
    </ErrorBoundary>
  </StrictMode>
);
