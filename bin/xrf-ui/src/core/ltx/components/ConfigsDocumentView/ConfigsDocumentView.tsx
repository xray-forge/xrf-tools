import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ConfigsDocument } from "@/core/ipc/types/xrf-app";
import { ConfigsAuthoredView } from "@/core/ltx/components/ConfigsDocumentView/ConfigsAuthoredView";
import { ConfigsResolvedView } from "@/core/ltx/components/ConfigsDocumentView/ConfigsResolvedView";
import { ConfigsDocumentService, EConfigsDocumentMode } from "@/core/ltx/services/document";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { inline } from "@/lib/callbacks/inline";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IConfigsDocumentViewProps extends BaseComponentProps {
  /** Ends the selection without closing the project, which the header offers. */
  onDeselect: () => void;
}

/**
 * The config on screen, in whichever of the two views is open.
 *
 * The states that belong to neither view live here - nothing selected, a read that failed - so each view below can
 * assume it has something to show. The header names the selection rather than the document: a read still in flight is
 * already a config somebody picked, and one that failed stays named so the retry beneath it says what it retries.
 */
export function ConfigsDocumentView({
  "data-testid": dataTestId = "configs-document-view",
  id,
  className,
  onDeselect,
}: IConfigsDocumentViewProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);

  const selected: Nullable<string> = documentService.selected;
  const document: Nullable<ConfigsDocument> = documentService.document.value;

  if (!selected) {
    return (
      <EmptyState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"No config open"}
        description={"Pick one in the tree to read what it says."}
      />
    );
  }

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}
    >
      <EditorFileHeader
        data-testid={"configs-document-header"}
        name={selected}
        closeLabel={"Close config"}
        closeDescription={"Clear the selection and close this config"}
        onClose={onDeselect}
      />

      <Box sx={{ display: "flex", flexGrow: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}>
        {inline(() => {
          if (documentService.document.isFailed) {
            return (
              <ErrorState
                title={"Cannot read this config"}
                description={documentService.document.error?.message ?? "The read failed."}
                onRetry={() => void documentService.retry()}
              />
            );
          }

          if (!document) {
            return <DelayedProgress />;
          }

          return documentService.mode === EConfigsDocumentMode.RESOLVED ? (
            <ConfigsResolvedView />
          ) : (
            <ConfigsAuthoredView document={document} />
          );
        })}
      </Box>
    </Box>
  );
}
