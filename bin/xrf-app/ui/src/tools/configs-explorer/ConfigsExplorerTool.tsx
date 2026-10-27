import { useInjection } from "@wirestate/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { ToolLoader } from "@/core/shell/loading/ToolLoader";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";

import { ConfigsExplorerOpenForm } from "./components/ConfigsExplorerOpenForm";
import { ConfigsExplorerWorkspace } from "./components/ConfigsExplorerWorkspace";

/**
 * Browse a tree of LTX configs, and read what each one says.
 */
export function ConfigsExplorerTool({
  "data-testid": dataTestId = "configs-explorer-tool",
}: BaseComponentProps): ReactElement {
  const projectService: ConfigsProjectService = useInjection(ConfigsProjectService);

  if (!projectService.project.isRestored) {
    return <ToolLoader />;
  }

  if (!projectService.isOpen) {
    return <ConfigsExplorerOpenForm data-testid={dataTestId} />;
  }

  return <ConfigsExplorerWorkspace data-testid={dataTestId} />;
}
