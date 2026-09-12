import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ArchivesEditorOpenForm } from "@/applications/archives-explorer/components/ArchivesEditorOpenForm";
import { ArchivesEditor } from "@/applications/archives-explorer/components/editor/ArchivesEditor";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";

export function ArchivesExplorerApplication(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  if (archivesService.project.isIdle) {
    return <ApplicationLoader />;
  }

  return archivesService.project.value ? <ArchivesEditor /> : <ArchivesEditorOpenForm />;
}
