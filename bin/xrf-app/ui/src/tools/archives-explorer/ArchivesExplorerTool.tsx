import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { ArchivesEditorOpenForm } from "@/tools/archives-explorer/components/ArchivesEditorOpenForm";
import { ArchivesEditor } from "@/tools/archives-explorer/components/editor/ArchivesEditor";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

export function ArchivesExplorerTool(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  return (
    <SessionTool root={archivesService.subject} openForm={<ArchivesEditorOpenForm />} workspace={<ArchivesEditor />} />
  );
}
