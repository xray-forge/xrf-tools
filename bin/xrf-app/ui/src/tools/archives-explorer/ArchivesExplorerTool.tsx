import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { ArchivesExplorerOpenForm } from "@/tools/archives-explorer/components/ArchivesExplorerOpenForm";
import { ArchivesExplorerWorkspace } from "@/tools/archives-explorer/components/ArchivesExplorerWorkspace";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

export function ArchivesExplorerTool(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  return (
    <SessionTool
      root={archivesService.subject}
      openForm={<ArchivesExplorerOpenForm />}
      workspace={<ArchivesExplorerWorkspace />}
    />
  );
}
