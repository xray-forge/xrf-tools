import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { LtxAnchoredFinding } from "@/core/ipc/types/xrf-ltx-inspect";
import { toOrderedFindings } from "@/core/ltx/lib/findings";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { EditorProblemsPanel, IEditorProblem, IEditorProblemLocation } from "@/core/shell/editor/EditorProblemsPanel";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * One finding as a row: what broke, where it is written, and where opening it leads.
 *
 * A finding the anchor could not place carries no location, and the row stays text rather than becoming a button to
 * nowhere - which is what a dialect diagnostic about a root rather than a config comes back as.
 *
 * @param finding - The finding to draw.
 * @returns The row.
 */
function toProblem(finding: LtxAnchoredFinding): IEditorProblem {
  return {
    location: finding.file ? { line: finding.line, path: finding.file } : undefined,
    message: finding.engineBehaviour ? `${finding.message}. ${finding.engineBehaviour}` : finding.message,
    rule: finding.section ? `${finding.kind}: ${finding.section}` : finding.kind,
    subject: finding.line === null ? finding.file : `${finding.file}:${finding.line}`,
  };
}

/**
 * Everything wrong with the root the open config belongs to.
 */
export function ConfigsProblemsPanel({
  "data-testid": dataTestId = "configs-problems-panel",
}: BaseComponentProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);
  const findingsService: ConfigsFindingsService = useInjection(ConfigsFindingsService);

  const entry: Nullable<string> = documentService.entry;
  const found: Nullable<Array<LtxAnchoredFinding>> = findingsService.findings.value;
  const own: Nullable<Array<LtxAnchoredFinding>> = documentService.document.value?.findings ?? null;

  const problems: Array<IEditorProblem> = useMemo(
    // The file's own findings are its parse failure and the includes that reached nothing, which verifying a root does
    // not raise; nothing is in both lists.
    () => toOrderedFindings([...(found ?? []), ...(own ?? [])]).map(toProblem),
    [found, own]
  );

  // The jump is the point of the panel: a finding names a config and a line, and both usually belong to a config that
  // is not the one on screen.
  const onSelect = useCallback(
    (location: IEditorProblemLocation) => void documentService.openAt(location.path, location.line ?? 1),
    [documentService]
  );

  // Asked for by this panel being on screen, which is what "on demand" means here.
  useEffect(() => {
    if (entry) {
      void findingsService.open(entry);
    }
  }, [entry, findingsService]);

  if (findingsService.findings.isFailed) {
    return (
      <ErrorState
        data-testid={dataTestId}
        title={"Cannot verify this root"}
        description={findingsService.findings.error?.message ?? "The verification failed."}
        onRetry={() => void (entry && findingsService.open(entry))}
      />
    );
  }

  if (entry && findingsService.findings.isLoading) {
    return <DelayedProgress data-testid={dataTestId} />;
  }

  return (
    <EditorProblemsPanel
      data-testid={dataTestId}
      findings={problems}
      emptyDescription={
        entry
          ? `${entry} resolves with nothing to report.`
          : "Nothing resolves this config, so there is no root to check it against."
      }
      onSelect={onSelect}
    />
  );
}
