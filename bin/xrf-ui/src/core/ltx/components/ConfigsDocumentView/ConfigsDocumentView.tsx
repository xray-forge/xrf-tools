import { Alert, Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { ConfigsDocument } from "@/core/bindings/types/xrf-app";
import { toDocumentLines } from "@/core/ltx/lib/semantic";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ICodeLine } from "@/core/ui/code/code-line";
import { VirtualizedLines } from "@/core/ui/code/VirtualizedLines";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * The config on screen: its text, coloured by what it says and by what resolving it found.
 */
export function ConfigsDocumentView({
  "data-testid": dataTestId = "configs-document-view",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);

  const document: Nullable<ConfigsDocument> = documentService.document.value;

  const lines: Array<ICodeLine> = useMemo(
    () => (document ? toDocumentLines(document.text.lines, document.structure) : []),
    [document]
  );

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

  return (
    <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}>
      {documentService.parseError ? (
        <Alert severity={"error"} variant={"outlined"} square>
          {`This config does not parse: ${documentService.parseError}. Its text is shown as written; nothing below ` +
            "reflects what the engine would load."}
        </Alert>
      ) : null}

      <VirtualizedLines
        data-testid={dataTestId}
        ariaLabel={`Contents of ${document.text.path}`}
        lines={lines}
        sx={{ flexGrow: 1, minWidth: 0, minHeight: 0 }}
      />
    </Box>
  );
}
