import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { ExportsService } from "@/tools/exports-explorer/services/exports.service";

import { ExportsOpenForm } from "./components/ExportsOpenForm";
import { ExportsEditor } from "./components/viewer/ExportsEditor";

/** Picker until a project is open, viewer once it is. */
export function ExportsExplorerTool(): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  return <SessionTool root={exportsService.project} openForm={<ExportsOpenForm />} workspace={<ExportsEditor />} />;
}
