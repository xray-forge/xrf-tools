import { Chip, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { selectRelatedApplications } from "@/core/help/lib/related";
import { EApplicationId, IApplicationDescriptor } from "@/core/routing/application";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpRelatedProps extends BaseComponentProps {
  relatedTools: ReadonlyArray<EApplicationId>;
  /** Called after a related tool is navigated to, so the hosting surface can dismiss itself. */
  onNavigated?: () => void;
}

/**
 * Sibling applications of the same workflow, as chips that go there.
 */
export function ApplicationHelpRelated({
  "data-testid": dataTestId = "application-help-related",
  id,
  className,
  relatedTools,
  onNavigated,
}: IApplicationHelpRelatedProps): Nullable<ReactElement> {
  const navigate: NavigateFunction = useNavigate();
  const applications: Array<IApplicationDescriptor> = selectRelatedApplications(relatedTools);

  return applications.length ? (
    <div data-testid={dataTestId} id={id} className={className}>
      <Typography className={"mb-1.5 text-text-primary"} variant={"subtitle2"}>
        Related tools
      </Typography>

      <div className={"flex flex-wrap gap-2"}>
        {applications.map((application: IApplicationDescriptor) => (
          <Chip
            key={application.id}
            className={"px-2 py-4"}
            icon={application.icon}
            label={application.label}
            color={"primary"}
            size={"small"}
            variant={"outlined"}
            onClick={() => {
              navigate(application.path);
              onNavigated?.();
            }}
          />
        ))}
      </div>
    </div>
  ) : null;
}
