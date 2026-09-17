import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { ApplicationHelpRelated } from "@/core/help/components/ApplicationHelpContent/ApplicationHelpRelated";
import { ApplicationHelpSection } from "@/core/help/components/ApplicationHelpContent/ApplicationHelpSection";
import { ApplicationHelpShortcuts } from "@/core/help/components/ApplicationHelpContent/ApplicationHelpShortcuts";
import { renderHelpText } from "@/core/help/lib/help-text";
import { IApplicationHelp } from "@/core/routing/application";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpContentProps extends BaseComponentProps {
  help: IApplicationHelp;
  /** Called after a related tool is navigated to, so the hosting surface can dismiss itself. */
  onNavigated?: () => void;
}

/**
 * The rubric body of one application's help, identical on every hosting surface.
 */
export function ApplicationHelpContent({
  "data-testid": dataTestId = "application-help-content",
  id,
  className,
  help,
  onNavigated,
}: IApplicationHelpContentProps): ReactElement {
  const sections: Array<ReactNode> = [];

  if (help.workflow?.length) {
    sections.push(
      <ApplicationHelpSection key={"workflow"} title={"Typical workflow"} items={help.workflow} isOrdered />
    );
  }

  if (help.nuances?.length) {
    sections.push(<ApplicationHelpSection key={"nuances"} title={"Nuances"} items={help.nuances} />);
  }

  if (help.limitations?.length) {
    sections.push(<ApplicationHelpSection key={"limitations"} title={"Limitations"} items={help.limitations} />);
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-4", className)}>
      <Typography variant={"body2"} sx={{ lineHeight: 1.55 }}>
        {renderHelpText(help.summary)}
      </Typography>

      {sections}

      <ApplicationHelpShortcuts />

      {help.relatedTools?.length ? (
        <ApplicationHelpRelated relatedTools={help.relatedTools} onNavigated={onNavigated} />
      ) : null}
    </div>
  );
}
