import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ExportsOpenForm } from "@/applications/exports-explorer/components/ExportsOpenForm";
import { ExportsEditor } from "@/applications/exports-explorer/components/viewer/ExportsEditor";
import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";

/** Picker until a project is open, viewer once it is. */
export function ExportsExplorerApplication(): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  if (exportsService.isReady) {
    return exportsService.project.value ? <ExportsEditor /> : <ExportsOpenForm />;
  }

  return <ApplicationLoader />;
}
