import { Logger } from "@xrf/logging";
import { ErrorBoundary } from "@xrf/react/error-boundary";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { ApplicationProvider } from "@/ApplicationProvider";
import { ApplicationRouter } from "@/ApplicationRouter";
import { RootCrash } from "@/core/error/components/RootCrash";
import { UI_LOG } from "@/core/logging/lib/ui-log";
import { tidyAppCacheOnStart } from "@/core/settings/lib/app-cache";
import { persistIpcProfiling } from "@/core/settings/lib/ipc-profiling";
import { suppressNativeContextMenu } from "@/lib/dom/event/context-menu";

// Before anything renders and calls the backend, so the first calls already know whether they weigh themselves.
persistIpcProfiling();

// Before the first line is logged, so the UI log file holds the whole run.
UI_LOG.install();

Logger.info("Start application");

// Set once for the lifetime of the webview rather than from a component: it is host chrome policy,
// not application state, and nothing ever restores the default menu.
suppressNativeContextMenu();

// Off the first paint's way: the backend lists the caches on its pool, at most once a day.
tidyAppCacheOnStart();

createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <ErrorBoundary fallback={RootCrash}>
      <ApplicationProvider>
        <ApplicationRouter />
      </ApplicationProvider>
    </ErrorBoundary>
  </StrictMode>
);
