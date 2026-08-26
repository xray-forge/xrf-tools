import { Box, Card, CardActionArea, Theme, Tooltip, Typography } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ApplicationLauncherGroupLabel } from "@/core/launcher/ApplicationLauncherGroupLabel";
import { ApplicationLauncherPlannedBadge } from "@/core/launcher/ApplicationLauncherPlannedBadge";
import { EApplicationStatus, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";

export interface IApplicationLauncherCardProps {
  application: IApplicationDescriptor;
  group: IApplicationGroup;
  isEnabled: boolean;
  /** Names the group on the card itself, for a body with no section heading above it to say so. */
  isGroupNamed?: boolean;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * One application on the root catalog grid.
 */
export function ApplicationLauncherCard({
  application,
  group,
  isEnabled,
  isGroupNamed,
  onOpen,
}: IApplicationLauncherCardProps): ReactElement {
  const isPlanned: boolean = application.status === EApplicationStatus.PLANNED;

  const onWarm = useCallback(() => {
    if (isEnabled) {
      void application.preload?.();
    }
  }, [application, isEnabled]);

  const content: ReactElement = (
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

        {isPlanned ? <ApplicationLauncherPlannedBadge /> : null}
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
  );

  return (
    <Card
      sx={{
        height: "100%",
        backgroundColor: "background.paper",
        transition: "background-color 140ms ease, border-color 140ms ease",
        ...(isEnabled
          ? {
              "&:hover": {
                backgroundColor: "action.hover",
                borderColor: "primary.main",
              },
            }
          : {
              backgroundColor: "transparent",
              borderStyle: "dashed",
            }),
      }}
    >
      {isEnabled ? (
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
          onClick={() => onOpen(application)}
        >
          {content}
        </CardActionArea>
      ) : (
        <Tooltip describeChild title={"Not implemented yet"}>
          <Box sx={{ height: "100%", cursor: "not-allowed" }}>{content}</Box>
        </Tooltip>
      )}
    </Card>
  );
}
