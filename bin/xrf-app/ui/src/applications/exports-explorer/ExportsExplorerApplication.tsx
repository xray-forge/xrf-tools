import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";

import { ExportsOpenForm } from "./components/ExportsOpenForm";
import { ExportsEditor } from "./components/viewer/ExportsEditor";

/** Picker until a project is open, viewer once it is. */
export function ExportsExplorerApplication(): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  if (exportsService.isReady) {
    return exportsService.project.value ? <ExportsEditor /> : <ExportsOpenForm />;
  }

  return <ApplicationLoader />;
}
