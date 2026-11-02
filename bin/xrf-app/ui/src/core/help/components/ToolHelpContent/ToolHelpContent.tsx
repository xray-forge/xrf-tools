import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode } from "react";

import { renderHelpText } from "@/core/help/lib/help-text";
import { IToolHelp } from "@/core/routing/tool";

import { ToolHelpRelated } from "./ToolHelpRelated";
import { ToolHelpSection } from "./ToolHelpSection";
import { ToolHelpShortcuts } from "./ToolHelpShortcuts";

export interface IToolHelpContentProps extends BaseComponentProps {
  help: IToolHelp;
  /** Called after a related tool is navigated to, so the hosting surface can dismiss itself. */
  onNavigated?: () => void;
}

/**
 * The rubric body of one tool's help, identical on every hosting surface.
 */
export function ToolHelpContent({
  "data-testid": dataTestId = "tool-help-content",
  id,
  className,
  help,
  onNavigated,
}: IToolHelpContentProps): ReactElement {
  const sections: Array<ReactNode> = [];

  if (help.workflow?.length) {
    sections.push(<ToolHelpSection key={"workflow"} title={"Typical workflow"} items={help.workflow} isOrdered />);
  }

  if (help.nuances?.length) {
    sections.push(<ToolHelpSection key={"nuances"} title={"Nuances"} items={help.nuances} />);
  }

  if (help.limitations?.length) {
    sections.push(<ToolHelpSection key={"limitations"} title={"Limitations"} items={help.limitations} />);
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-4", className)}>
      <Typography className={"leading-panel"} variant={"body2"}>
        {renderHelpText(help.summary)}
      </Typography>

      {sections}

      <ToolHelpShortcuts />

      {help.relatedTools?.length ? (
        <ToolHelpRelated relatedTools={help.relatedTools} onNavigated={onNavigated} />
      ) : null}
    </div>
  );
}
