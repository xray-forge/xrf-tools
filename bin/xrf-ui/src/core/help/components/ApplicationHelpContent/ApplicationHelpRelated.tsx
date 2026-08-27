import { Box, Chip, Typography } from "@mui/material";
import { ReactElement } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { selectRelatedApplications } from "@/core/help/lib/related";
import { EApplicationId, IApplicationDescriptor } from "@/core/routing/application";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
  sx,
  relatedTools,
  onNavigated,
}: IApplicationHelpRelatedProps): Nullable<ReactElement> {
  const navigate: NavigateFunction = useNavigate();
  const applications: Array<IApplicationDescriptor> = selectRelatedApplications(relatedTools);

  return applications.length ? (
    <Box data-testid={dataTestId} id={id} className={className} sx={sx}>
      <Typography variant={"subtitle2"} sx={{ color: "text.primary", marginBottom: 0.75 }}>
        Related tools
      </Typography>

      <Box sx={{ display: "flex", flexWrap: "wrap", gap: 1 }}>
        {applications.map((application: IApplicationDescriptor) => (
          <Chip
            key={application.id}
            icon={application.icon}
            label={application.label}
            size={"small"}
            variant={"outlined"}
            sx={{ paddingX: 1, paddingY: 2 }}
            onClick={() => {
              navigate(application.path);
              onNavigated?.();
            }}
          />
        ))}
      </Box>
    </Box>
  ) : null;
}
