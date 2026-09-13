import { Box, Card, CardActionArea, Theme, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ApplicationLauncherGroupLabel } from "@/core/launcher/components/ApplicationLauncherGroupLabel";
import { ApplicationLauncherPlannedBadge } from "@/core/launcher/components/ApplicationLauncherPlannedBadge";
import { useApplicationLauncherActions } from "@/core/launcher/lib";
import { EApplicationStatus, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IApplicationLauncherCardProps extends BaseComponentProps {
  application: IApplicationDescriptor;
  group: IApplicationGroup;
  /** Names the group on the card itself, for a body with no section heading above it to say so. */
  isGroupNamed?: boolean;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * One application on the root catalog grid.
 */
export function ApplicationLauncherCard({
  "data-testid": dataTestId = "application-launcher-card",
  id,
  className,
  application,
  group,
  isGroupNamed,
  onOpen,
}: IApplicationLauncherCardProps): ReactElement {
  const { onWarm, onClick } = useApplicationLauncherActions(application, onOpen);

  return (
    <Card data-testid={dataTestId} id={id} className={className} sx={{ height: "100%" }}>
      <CardActionArea
        aria-label={application.label}
        sx={{
          display: "block",
          height: "100%",
          "&.Mui-focusVisible": {
            outline: "2px solid",
            outlineColor: "primary.main",
            outlineOffset: -2,
          },
        }}
        onFocus={onWarm}
        onMouseEnter={onWarm}
        onClick={onClick}
      >
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            gap: 0.5,
            height: "100%",
            padding: 1.25,
          }}
        >
          <Box sx={{ display: "flex", alignItems: "center", gap: 0.75, minWidth: 0 }}>
            <Box
              aria-hidden={true}
              sx={(theme: Theme) => ({
                display: "flex",
                flexShrink: 0,
                color: group.accent.light,
                "& .MuiSvgIcon-root": { fontSize: 18 },
                ...theme.applyStyles("dark", { color: group.accent.dark }),
              })}
            >
              {application.icon}
            </Box>

            <Typography
              variant={"subtitle2"}
              sx={{
                display: "-webkit-box",
                flexGrow: 1,
                minWidth: 0,
                color: "text.primary",
                WebkitBoxOrient: "vertical",
                WebkitLineClamp: 2,
                overflow: "hidden",
                lineHeight: 1.3,
              }}
            >
              {application.label}
            </Typography>

            {application.status === EApplicationStatus.PLANNED ? <ApplicationLauncherPlannedBadge /> : null}
          </Box>

          <Typography
            variant={"body2"}
            sx={{
              display: "-webkit-box",
              color: "text.secondary",
              WebkitBoxOrient: "vertical",
              WebkitLineClamp: 2,
              overflow: "hidden",
              lineHeight: 1.35,
            }}
          >
            {application.description}
          </Typography>

          {isGroupNamed ? <ApplicationLauncherGroupLabel group={group} /> : null}
        </Box>
      </CardActionArea>
    </Card>
  );
}
