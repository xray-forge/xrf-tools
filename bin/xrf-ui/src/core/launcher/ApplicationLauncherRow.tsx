import { Box, ListItem, ListItemButton, Theme, Tooltip, Typography } from "@mui/material";
import { ReactElement, ReactNode, useCallback } from "react";

import { ApplicationLauncherGroupLabel } from "@/core/launcher/ApplicationLauncherGroupLabel";
import { EApplicationStatus, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { TREE } from "@/core/theme/tokens";

/** A row measures the same as an explorer tree row, so the two read as one application. */
const ROW_LAYOUT = {
  display: "grid",
  gridTemplateColumns: `${TREE.iconWidth}px 240px minmax(0, 1fr) 132px 72px`,
  alignItems: "center",
  gap: 1,
  height: TREE.rowHeight,
  paddingX: 1,
  paddingY: 0,
} as const;

export interface IApplicationLauncherRowProps {
  application: IApplicationDescriptor;
  group: IApplicationGroup;
  isEnabled: boolean;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * One application on the root catalog list.
 */
export function ApplicationLauncherRow({
  application,
  group,
  isEnabled,
  onOpen,
}: IApplicationLauncherRowProps): ReactElement {
  const isPlanned: boolean = application.status === EApplicationStatus.PLANNED;

  const onWarm = useCallback(() => {
    if (isEnabled) {
      // Nothing awaits this: the point is only that the fetch has started before the click.
      void application.preload?.();
    }
  }, [application, isEnabled]);

  const content: ReactNode = (
    <>
      <Box
        aria-hidden={true}
        sx={(theme: Theme) => ({
          display: "flex",
          flexShrink: 0,
          color: group.accent.light,
          "& .MuiSvgIcon-root": { fontSize: TREE.iconSize },
          ...theme.applyStyles("dark", { color: group.accent.dark }),
        })}
      >
        {application.icon}
      </Box>

      <Typography
        variant={"body2"}
        sx={{
          minWidth: 0,
          overflow: "hidden",
          color: "text.primary",
          fontWeight: 500,
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
      >
        {application.label}
      </Typography>

      <Typography
        variant={"body2"}
        sx={{
          minWidth: 0,
          overflow: "hidden",
          color: "text.secondary",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
      >
        {application.description}
      </Typography>

      <ApplicationLauncherGroupLabel group={group} />

      <Box sx={{ display: "flex", minWidth: 0 }}>
        {isPlanned ? (
          <Typography
            component={"span"}
            variant={"caption"}
            sx={{
              paddingX: 0.75,
              color: "text.secondary",
              border: "1px solid",
              borderColor: "divider",
              borderRadius: 1,
              fontSize: "0.625rem",
              fontWeight: 600,
              lineHeight: "17px",
            }}
          >
            Planned
          </Typography>
        ) : null}
      </Box>
    </>
  );

  return (
    <ListItem disablePadding={true} sx={{ display: "block" }}>
      {isEnabled ? (
        <ListItemButton
          aria-label={application.label}
          sx={ROW_LAYOUT}
          onFocus={onWarm}
          onMouseEnter={onWarm}
          onClick={() => onOpen(application)}
        >
          {content}
        </ListItemButton>
      ) : (
        <Tooltip describeChild title={"Not implemented yet"}>
          <Box sx={{ ...ROW_LAYOUT, cursor: "not-allowed" }}>{content}</Box>
        </Tooltip>
      )}
    </ListItem>
  );
}
