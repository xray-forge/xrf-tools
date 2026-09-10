import { Alert, Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { ConfigsDocument } from "@/core/bindings/types/xrf-app";
import { useRevealedSection } from "@/core/ltx/components/ConfigsDocumentView/use-revealed-section";
import { toDocumentLines } from "@/core/ltx/lib/semantic";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ICodeLineSource, toCodeLineSource } from "@/core/ui/code/code-line";
import { VirtualizedLines } from "@/core/ui/code/VirtualizedLines";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IConfigsAuthoredViewProps extends BaseComponentProps {
  document: ConfigsDocument;
}

/**
 * The open config as written, coloured by what it says and by what resolving it found.
 */
export function ConfigsAuthoredView({
  "data-testid": dataTestId = "configs-authored-view",
  document,
}: IConfigsAuthoredViewProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);

  const revealed: Nullable<string> = useRevealedSection();

  // Held whole rather than built a line at a time: a config as authored is thousands of lines, and each is coloured
  // from text the backend has already sent.
  const source: ICodeLineSource = useMemo(
    () => toCodeLineSource(toDocumentLines(document.text.lines, document.structure)),
    [document]
  );

  // Sections carry their own line here, because this view shows the file rather than a resolution of it.
  const revealedLine: Nullable<number> = useMemo(
    () => document.structure.sections.find((section) => section.name === revealed)?.line ?? null,
    [document, revealed]
  );

  return (
    <Box
      data-testid={dataTestId}
      sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}
    >
      {documentService.parseError ? (
        <Alert severity={"error"} variant={"outlined"} square>
          {`This config does not parse: ${documentService.parseError}. Its text is shown as written; nothing below ` +
            "reflects what the engine would load."}
        </Alert>
      ) : null}

      <VirtualizedLines
        data-testid={"configs-authored-lines"}
        ariaLabel={`Contents of ${document.text.path}`}
        source={source}
        scrollToLine={revealedLine}
        sx={{ flexGrow: 1, minWidth: 0, minHeight: 0 }}
      />
    </Box>
  );
}
