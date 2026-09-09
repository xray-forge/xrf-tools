import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { ConfigsExplorerOpenForm } from "@/applications/configs-explorer/components/ConfigsExplorerOpenForm";
import { ConfigsExplorerWorkspace } from "@/applications/configs-explorer/components/ConfigsExplorerWorkspace";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IConfigsExplorerApplicationProps extends BaseComponentProps {}

/**
 * Browse a tree of LTX configs, and read what each one says.
 */
export function ConfigsExplorerApplication({
  "data-testid": dataTestId = "configs-explorer-application",
}: IConfigsExplorerApplicationProps): ReactElement {
  const projectService: ConfigsProjectService = useInjection(ConfigsProjectService);

  const [isPickerOpen, setPickerOpen] = useState<boolean>(false);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  if (!projectService.isReady) {
    return <ApplicationLoader />;
  }

  if (isPickerOpen || !projectService.isOpen) {
    return <ConfigsExplorerOpenForm data-testid={dataTestId} onFinished={onFinished} />;
  }

  return <ConfigsExplorerWorkspace data-testid={dataTestId} />;
}
