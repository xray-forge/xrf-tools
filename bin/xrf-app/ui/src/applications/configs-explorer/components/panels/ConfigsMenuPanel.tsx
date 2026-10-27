import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ConfigsMenu } from "@/applications/configs-explorer/components/ConfigsMenu";
import { ConfigsDocumentService } from "@/applications/configs-explorer/services/configs-document";
import { ConfigsProjectService } from "@/applications/configs-explorer/services/configs-project";

/**
 * The project tree, wired to whichever config is on screen.
 */
export function ConfigsMenuPanel(): ReactElement {
  const projectService: ConfigsProjectService = useInjection(ConfigsProjectService);
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);

  const onOpen = useCallback((path: string) => void documentService.select(path), [documentService]);

  return <ConfigsMenu files={projectService.files} selected={documentService.selected} onOpen={onOpen} />;
}
