import { Box } from "@mui/material";
import { ReactElement, ReactNode } from "react";
import { createPortal } from "react-dom";

import { useEditorToolbarHost } from "@/core/shell/header/editor-toolbar-host";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IEditorLayoutProps extends BaseComponentProps {
  /** Portaled into the window caption, so it must stay one row of controls. */
  toolbar?: ReactNode;
  /** Full-width notices between the caption and the content. */
  banner?: ReactNode;
  footer?: ReactNode;
  children?: ReactNode;
}

/**
 * Workspace shell shared by every application.
 */
export function EditorLayout({
  "data-testid": dataTestId,
  id,
  className,
  toolbar,
  banner,
  footer,
  children,
}: IEditorLayoutProps): ReactElement {
  const host: Nullable<HTMLElement> = useEditorToolbarHost();

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexDirection: "column", width: "100%", height: "100%", flexWrap: "nowrap" }}
    >
      {toolbar && host ? createPortal(toolbar, host) : toolbar}

      {banner ? <Box sx={{ flexShrink: 0 }}>{banner}</Box> : null}

      <Box sx={{ display: "flex", flexGrow: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}>{children}</Box>

      {footer}
    </Box>
  );
}
