import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ToolLoader } from "@/core/shell/loading/ToolLoader";
import { ExportsService } from "@/tools/exports-explorer/services/exports.service";

import { ExportsOpenForm } from "./components/ExportsOpenForm";
import { ExportsEditor } from "./components/viewer/ExportsEditor";

/** Picker until a project is open, viewer once it is. */
export function ExportsExplorerTool(): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  if (exportsService.project.isRestored) {
    return exportsService.project.value ? <ExportsEditor /> : <ExportsOpenForm />;
  }

  return <ToolLoader />;
}
