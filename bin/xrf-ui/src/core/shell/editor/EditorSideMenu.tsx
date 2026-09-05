import { Box, Divider, List, ListItem, ListItemButton, ListItemIcon, ListItemText } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { mergeSx } from "@/core/theme/merge-sx";
import { StyledComponentProps } from "@/lib/dom/element-types";

export interface IEditorSideMenuItem {
  label: string;
  icon?: ReactNode;
  description?: string;
  isSelected?: boolean;
  isDisabled?: boolean;
  onClick?: () => void;
}

interface IEditorSideMenuProps extends StyledComponentProps {
  header?: ReactNode;
  sections?: Array<IEditorSideMenuItem>;
  actions?: Array<IEditorSideMenuItem>;
  /** Pinned directly above the actions, for whatever the last one of them has to report. */
  footer?: ReactNode;
  children?: ReactNode;
}

function renderItem(item: IEditorSideMenuItem): ReactElement {
  return (
    <ListItem key={item.label} disablePadding>
      <ListItemButton selected={item.isSelected} disabled={item.isDisabled} onClick={item.onClick}>
        {item.icon ? <ListItemIcon sx={{ minWidth: 40 }}>{item.icon}</ListItemIcon> : null}
        <ListItemText primary={item.label} secondary={item.description} />
      </ListItemButton>
    </ListItem>
  );
}

/**
 * Body of a navigation panel: an optional header, a scrolling middle that is either a section list or
 * arbitrary content, and actions pinned to the bottom.
 */
export function EditorSideMenu({
  "data-testid": dataTestId = "editor-side-menu",
  id,
  className,
  sx,
  header,
  sections,
  actions,
  footer,
  children,
}: IEditorSideMenuProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={mergeSx({ display: "flex", flexDirection: "column", width: "100%", height: "100%", minHeight: 0 }, sx)}
    >
      {header ? <Box sx={{ flexShrink: 0 }}>{header}</Box> : null}

      <Box sx={{ flexGrow: 1, minHeight: 0, overflowY: "auto" }}>
        {sections?.length ? <List disablePadding>{sections.map(renderItem)}</List> : null}
        {children}
      </Box>

      {footer ? <Box sx={{ flexShrink: 0 }}>{footer}</Box> : null}

      {actions?.length ? (
        <>
          <Divider />
          <List disablePadding sx={{ flexShrink: 0 }}>
            {actions.map(renderItem)}
          </List>
        </>
      ) : null}
    </Box>
  );
}
