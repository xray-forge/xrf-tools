import { Box, ListItem, ListItemButton, Theme, Tooltip, Typography } from "@mui/material";
import { ReactElement, ReactNode, useCallback } from "react";

import { ApplicationLauncherGroupLabel } from "@/core/launcher/ApplicationLauncherGroupLabel";
import { ApplicationLauncherPlannedBadge } from "@/core/launcher/ApplicationLauncherPlannedBadge";
import { EApplicationStatus, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { TREE } from "@/core/theme/tokens";

export interface IApplicationLauncherRowProps {
  application: IApplicationDescriptor;
  group: IApplicationGroup;
  isEnabled: boolean;
  /** Names the group on the row itself, for a list with no separator above it to say so. */
  isGroupNamed?: boolean;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * One application on the root catalog list.
 */
export function ApplicationLauncherRow({
  application,
  group,
  isEnabled,
  isGroupNamed,
  onOpen,
}: IApplicationLauncherRowProps): ReactElement {
  const isPlanned: boolean = application.status === EApplicationStatus.PLANNED;

  const onWarm = useCallback(() => {
    if (isEnabled) {
      // Nothing awaits this: the point is only that the fetch has started before the click.
      void application.preload?.();
    }
  }, [application, isEnabled]);

  /**
   * A row measures the same as an explorer tree row, so the two read as one application. The group
   * takes a column only where it is not already stated above the run this row belongs to.
   */
  const layout = {
    display: "grid",
    gridTemplateColumns: `${TREE.iconWidth}px 240px minmax(0, 1fr)${isGroupNamed ? " 132px" : ""}`,
    alignItems: "center",
    gap: 1,
    height: TREE.rowHeight,
    paddingX: 1,
    paddingY: 0,
  } as const;

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

      <Box sx={{ display: "flex", alignItems: "center", gap: 0.75, minWidth: 0 }}>
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

        {isPlanned ? <ApplicationLauncherPlannedBadge /> : null}
      </Box>

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

      {isGroupNamed ? <ApplicationLauncherGroupLabel group={group} /> : null}
    </>
  );

  return (
    <ListItem disablePadding={true} sx={{ display: "block" }}>
      {isEnabled ? (
        <ListItemButton
          aria-label={application.label}
          sx={layout}
          onFocus={onWarm}
          onMouseEnter={onWarm}
          onClick={() => onOpen(application)}
        >
          {content}
        </ListItemButton>
      ) : (
        <Tooltip describeChild title={"Not implemented yet"}>
          <Box sx={{ ...layout, cursor: "not-allowed" }}>{content}</Box>
        </Tooltip>
      )}
    </ListItem>
  );
}
