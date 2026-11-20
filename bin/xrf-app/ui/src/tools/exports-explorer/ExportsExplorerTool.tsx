import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { ExportsService } from "@/tools/exports-explorer/services/exports.service";

import { ExportsExplorerOpenForm } from "./components/ExportsExplorerOpenForm";
import { ExportsExplorerWorkspace } from "./components/ExportsExplorerWorkspace";

/** Picker until a project is open, viewer once it is. */
export function ExportsExplorerTool(): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  return (
    <SessionTool
      root={exportsService.project}
      openForm={<ExportsExplorerOpenForm />}
      workspace={<ExportsExplorerWorkspace />}
    />
  );
}
