import { Logger } from "@xrf/logging";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { ApplicationProvider } from "@/ApplicationProvider";
import { ApplicationRouter } from "@/ApplicationRouter";
import { ErrorBoundary } from "@/core/error/components/ErrorBoundary";
import { RootCrash } from "@/core/error/components/RootCrash";
import { tidyAppCacheOnStart } from "@/core/settings/lib/app-cache";
import { persistIpcProfiling } from "@/core/settings/lib/ipc-profiling";
import { suppressNativeContextMenu } from "@/lib/dom/event/context-menu";

Logger.info("Start application");

// Before anything renders and calls the backend, so the first calls already know whether they weigh themselves.
persistIpcProfiling();

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
