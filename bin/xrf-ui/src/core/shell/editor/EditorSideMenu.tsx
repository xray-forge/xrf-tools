import { Divider, List, ListItem, ListItemButton, ListItemIcon, ListItemText } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IEditorSideMenuItem {
  label: string;
  icon?: ReactNode;
  description?: string;
  isSelected?: boolean;
  isDisabled?: boolean;
  onClick?: () => void;
}

interface IEditorSideMenuProps extends BaseComponentProps {
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
  header,
  sections,
  actions,
  footer,
  children,
}: IEditorSideMenuProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex h-full min-h-0 w-full flex-col", className)}>
      {header ? <div className={"shrink-0"}>{header}</div> : null}

      <div className={"min-h-0 grow"} style={{ overflowY: "auto" }}>
        {sections?.length ? <List disablePadding>{sections.map(renderItem)}</List> : null}
        {children}
      </div>

      {footer ? <div className={"shrink-0"}>{footer}</div> : null}

      {actions?.length ? (
        <>
          <Divider />
          <List disablePadding sx={{ flexShrink: 0 }}>
            {actions.map(renderItem)}
          </List>
        </>
      ) : null}
    </div>
  );
}
