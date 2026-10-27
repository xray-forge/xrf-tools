import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ToolLoader } from "@/core/shell/chrome/ToolLoader";
import { ArchivesEditorOpenForm } from "@/tools/archives-explorer/components/ArchivesEditorOpenForm";
import { ArchivesEditor } from "@/tools/archives-explorer/components/editor/ArchivesEditor";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

export function ArchivesExplorerTool(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  if (!archivesService.subject.isRestored) {
    return <ToolLoader />;
  }

  return archivesService.subject.value ? <ArchivesEditor /> : <ArchivesEditorOpenForm />;
}
