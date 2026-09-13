import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ConfigsDocument } from "@/core/ipc/types/xrf-app";
import { ConfigsAuthoredView } from "@/core/ltx/components/ConfigsDocumentView/ConfigsAuthoredView";
import { ConfigsResolvedView } from "@/core/ltx/components/ConfigsDocumentView/ConfigsResolvedView";
import { ConfigsDocumentService, EConfigsDocumentMode } from "@/core/ltx/services/document";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * The config on screen, in whichever of the two views is open.
 *
 * The states that belong to neither view live here - nothing selected, a read that failed - so each view below can
 * assume it has something to show.
 */
export function ConfigsDocumentView({
  "data-testid": dataTestId = "configs-document-view",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);

  const document: Nullable<ConfigsDocument> = documentService.document.value;

  if (documentService.document.isFailed) {
    return (
      <ErrorState
        data-testid={dataTestId}
        title={"Cannot read this config"}
        description={documentService.document.error?.message ?? "The read failed."}
        onRetry={() => void documentService.retry()}
      />
    );
  }

  if (!document) {
    return (
      <EmptyState
        data-testid={dataTestId}
        title={"No config open"}
        description={"Pick one in the tree to read what it says."}
      />
    );
  }

  return documentService.mode === EConfigsDocumentMode.RESOLVED ? (
    <ConfigsResolvedView data-testid={dataTestId} />
  ) : (
    <ConfigsAuthoredView data-testid={dataTestId} document={document} />
  );
}
