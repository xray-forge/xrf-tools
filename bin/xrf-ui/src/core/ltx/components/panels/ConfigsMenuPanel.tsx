import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ConfigsMenu } from "@/core/ltx/components/ConfigsMenu";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";

/**
 * The project tree, wired to whichever config is on screen.
 */
export function ConfigsMenuPanel(): ReactElement {
  const projectService: ConfigsProjectService = useInjection(ConfigsProjectService);
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);

  const onOpen = useCallback((path: string) => void documentService.select(path), [documentService]);

  return <ConfigsMenu files={projectService.files} selected={documentService.selected} onOpen={onOpen} />;
}
