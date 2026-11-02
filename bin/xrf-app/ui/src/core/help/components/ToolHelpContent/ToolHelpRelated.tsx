import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material/chip";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { selectRelatedTools } from "@/core/help/lib/related";
import { EToolId, IToolDescriptor } from "@/core/routing/tool";

export interface IToolHelpRelatedProps extends BaseComponentProps {
  relatedTools: ReadonlyArray<EToolId>;
  /** Called after a related tool is navigated to, so the hosting surface can dismiss itself. */
  onNavigated?: () => void;
}

/**
 * Sibling tools of the same workflow, as chips that go there.
 */
export function ToolHelpRelated({
  "data-testid": dataTestId = "tool-help-related",
  id,
  className,
  relatedTools,
  onNavigated,
}: IToolHelpRelatedProps): Nullable<ReactElement> {
  const navigate: NavigateFunction = useNavigate();
  const tools: Array<IToolDescriptor> = selectRelatedTools(relatedTools);

  return tools.length ? (
    <div data-testid={dataTestId} id={id} className={className}>
      <Typography className={"mb-1.5 text-text-primary"} variant={"subtitle2"}>
        Related tools
      </Typography>

      <div className={"flex flex-wrap gap-2"}>
        {tools.map((tool: IToolDescriptor) => (
          <Chip
            key={tool.id}
            className={"px-2 py-4"}
            icon={tool.icon}
            label={tool.label}
            color={"primary"}
            size={"small"}
            variant={"outlined"}
            onClick={() => {
              navigate(tool.path);
              onNavigated?.();
            }}
          />
        ))}
      </div>
    </div>
  ) : null;
}
