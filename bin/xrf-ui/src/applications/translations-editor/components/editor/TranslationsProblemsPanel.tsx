import { ReactElement } from "react";

import { TranslationFinding } from "@/core/bindings/types/xrf-translation";
import { EditorProblemsPanel } from "@/core/shell/editor/EditorProblemsPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ITranslationsProblemsPanelProps extends BaseComponentProps {
  findings: ReadonlyArray<TranslationFinding>;
}

/**
 * Problems reported while reading the application's project.
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
