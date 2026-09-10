import { Alert, Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { ConfigsDocument } from "@/core/bindings/types/xrf-app";
import { LtxAnchoredFinding, LtxStructureSection } from "@/core/bindings/types/xrf-ltx-inspect";
import { useRevealed } from "@/core/ltx/components/ConfigsDocumentView/use-revealed";
import { toFindingMarks } from "@/core/ltx/lib/findings";
import { TConfigsReveal } from "@/core/ltx/lib/reveal";
import { toDocumentLines } from "@/core/ltx/lib/semantic";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ECodeLineMark, ICodeLine, ICodeLineSource, toCodeLineSource } from "@/core/ui/code/code-line";
import { VirtualizedLines } from "@/core/ui/code/VirtualizedLines";
import { EmptyState } from "@/core/ui/layout/EmptyState";
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
  const findingsService: ConfigsFindingsService = useInjection(ConfigsFindingsService);

  const revealed: Nullable<TConfigsReveal> = useRevealed();

  // The root's findings, which arrive when the Problems panel asks for them - usually after this config was opened.
  // Depended on by value rather than through the service, or the gutter of the file on screen would keep the marks it
  // had at open.
  const found: Nullable<Array<LtxAnchoredFinding>> = findingsService.findings.value;

  // The file's own findings and the root's, drawn from one list: a gutter with two sources has two answers about the
  // same line. `toFindingMarks` keeps only what belongs to this config.
  const marks: ReadonlyMap<number, ECodeLineMark> = useMemo(
    () => toFindingMarks([...document.findings, ...(found ?? [])], document.text.path),
    [document, found]
  );

  // Held whole rather than built a line at a time: a config as authored is thousands of lines, and each is coloured
  // from text the backend has already sent.
  const source: ICodeLineSource = useMemo(
    () => toCodeLineSource(toDocumentLines(document.text.lines, document.structure, marks)),
    [document, marks]
  );

  // Sections carry their own line here, because this view shows the file rather than a resolution of it.
  const revealedLine: Nullable<number> = useMemo(() => {
    if (!revealed) {
      return null;
    }

    return revealed.kind === "line"
      ? revealed.line
      : (document.structure.sections.find((section) => section.name === revealed.section)?.line ?? null);
  }, [document, revealed]);

  // The section a click landed in: the last header at or above it, which is what the engine would call that line's
  // section too. A click above the first header is in none.
  const onSelectLine = useCallback(
    (line: ICodeLine) => {
      const holding: Nullable<LtxStructureSection> = document.structure.sections.reduce<Nullable<LtxStructureSection>>(
        (found, section) => (section.line <= line.number ? section : found),
        null
      );

      documentService.selectSection(holding?.name ?? null);
    },
    [document, documentService]
  );

  if (!source.count) {
    return (
      <EmptyState
        data-testid={dataTestId}
        title={"This config is empty"}
        description={`${document.text.path} holds no text, so it declares nothing and adds nothing to whatever includes it.`}
      />
    );
  }

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
        onSelectLine={onSelectLine}
      />
    </Box>
  );
}
