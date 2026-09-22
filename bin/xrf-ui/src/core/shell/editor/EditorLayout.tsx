import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode } from "react";
import { createPortal } from "react-dom";

import { useEditorToolbarHost } from "@/core/shell/header/editor-toolbar-host";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex h-full w-full flex-col flex-nowrap surface-content", className)}
    >
      {toolbar && host ? createPortal(toolbar, host) : toolbar}

      {banner ? <div className={"shrink-0"}>{banner}</div> : null}

      <div className={"flex min-h-0 min-w-0 grow overflow-hidden"}>{children}</div>

      {footer}
    </div>
  );
}
