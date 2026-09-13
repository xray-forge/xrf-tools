import { Box, ListItem, ListItemButton, Theme, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ApplicationLauncherGroupLabel } from "@/core/launcher/components/ApplicationLauncherGroupLabel";
import { ApplicationLauncherPlannedBadge } from "@/core/launcher/components/ApplicationLauncherPlannedBadge";
import { useApplicationLauncherActions } from "@/core/launcher/lib";
import { EApplicationStatus, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { TREE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IApplicationLauncherRowProps extends BaseComponentProps {
  application: IApplicationDescriptor;
  group: IApplicationGroup;
  /** Names the group on the row itself, for a list with no separator above it to say so. */
  isGroupNamed?: boolean;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * One application on the root catalog list.
 */
export function ApplicationLauncherRow({
  "data-testid": dataTestId = "application-launcher-row",
  id,
  className,
  application,
  group,
  isGroupNamed,
  onOpen,
}: IApplicationLauncherRowProps): ReactElement {
  const { onWarm, onClick } = useApplicationLauncherActions(application, onOpen);

  return (
    <ListItem data-testid={dataTestId} id={id} className={className} disablePadding={true} sx={{ display: "block" }}>
      <ListItemButton
        aria-label={application.label}
        sx={{
          // A row measures the same as an explorer tree row, so the two read as one application. The group
          // takes a column only where it is not already stated above the section this row belongs to.
          display: "grid",
          gridTemplateColumns: `${TREE.iconWidth}px 240px minmax(0, 1fr)${isGroupNamed ? " 132px" : ""}`,
          alignItems: "center",
          gap: 1,
          height: TREE.rowHeight,
          paddingX: 1,
          paddingY: 0,
        }}
        onFocus={onWarm}
        onMouseEnter={onWarm}
        onClick={onClick}
      >
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

          {application.status === EApplicationStatus.PLANNED ? <ApplicationLauncherPlannedBadge /> : null}
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
      </ListItemButton>
    </ListItem>
  );
}
