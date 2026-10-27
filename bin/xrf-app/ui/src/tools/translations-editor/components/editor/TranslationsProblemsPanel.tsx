import { TranslationFinding } from "@xrf/ipc/types/xrf-translation";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { EditorProblemsPanel } from "@/core/shell/editor/EditorProblemsPanel";

export interface ITranslationsProblemsPanelProps extends BaseComponentProps {
  findings: ReadonlyArray<TranslationFinding>;
}

/**
 * Problems reported while reading the tool's project.
 */
export function TranslationsProblemsPanel({
  "data-testid": dataTestId = "translations-editor-problems-panel",
  id,
  className,
  findings,
}: ITranslationsProblemsPanelProps): ReactElement {
  return (
    <EditorProblemsPanel
      data-testid={dataTestId}
      id={id}
      className={className}
      findings={findings}
      rulePrefix={"translations."}
      emptyDescription={"Every file in this project read cleanly."}
    />
  );
}
