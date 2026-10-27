import { Nullable } from "@xrf/types";
import { EditorToolbarCrumb, EditorToolbarPathSeparator, EditorToolbarSeparator } from "@xrf/ui/editor/toolbar";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Box } from "@xrf/ui/material/box";
import { iconButtonClasses } from "@xrf/ui/material/icon-button";
import { svgIconClasses } from "@xrf/ui/material/svg-icon";
import { cn, CONTROL } from "@xrf/ui/theme";
import { ReactElement, ReactNode, useCallback } from "react";

import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";
import { useIsToolBusy, useRequestLeave } from "@/core/shell/lifecycle";

interface IEditorToolbarProps extends BaseComponentProps {
  /** Overrides the tool name resolved from the route. Rarely needed. */
  title?: string;
  /** The open document, as the last breadcrumb segment. Counts and state belong in the status bar. */
  subtitle?: ReactNode;
  actions?: ReactNode;
  /** Closes the document after the shared leave guard allows it. Omit it to make the segment inert. */
  onBack?: () => void;
}

/**
 * The active tool's row inside the window caption.
 */
export function EditorToolbar({
  "data-testid": dataTestId = "editor-toolbar",
  id = "editor-toolbar",
  className,
  title,
  subtitle,
  actions,
  onBack,
}: IEditorToolbarProps): ReactElement {
  const tool: Nullable<IToolDescriptor> = useCurrentTool();

  const isBusy: boolean = useIsToolBusy();
  const requestLeave: (leave: () => void) => void = useRequestLeave();

  const label: Nullable<string> = title ?? tool?.label ?? null;

  const hasActions: boolean = Boolean(actions);

  const onGoBack = useCallback(() => {
    if (onBack) {
      requestLeave(onBack);
    }
  }, [onBack, requestLeave]);

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex h-full w-full min-w-0 items-center gap-1.5", className)}>
      {/* Hidden under the window's open menubar, which lies over it, keeping its room so nothing beside it moves. */}
      <div className={"flex min-w-0 items-center gap-1.5 group-has-[[role=menubar]]/caption:invisible"}>
        {label ? (
          <EditorToolbarCrumb
            label={label}
            isDisabled={isBusy}
            accessibleName={onBack ? `Back to ${label}` : undefined}
            hint={onBack ? `Back to ${label}, closing what is open` : undefined}
            onClick={onBack ? onGoBack : undefined}
          />
        ) : null}

        {subtitle ? (
          <>
            <EditorToolbarPathSeparator />

            <Box
              className={"min-w-0 truncate text-left opacity-70"}
              sx={{ direction: "rtl", fontSize: "0.75rem", "& > *": { direction: "ltr" } }}
            >
              {subtitle}
            </Box>
          </>
        ) : null}
      </div>

      <div className={"min-w-2 grow"} />

      {hasActions ? (
        <>
          <Box
            data-testid={"editor-toolbar-actions"}
            className={"flex shrink-0 items-center"}
            sx={{
              [`& .${iconButtonClasses.root}`]: {
                width: CONTROL.editorActionSize,
                height: CONTROL.editorActionSize,
                padding: 0,
              },
              [`& .${svgIconClasses.root}`]: { fontSize: CONTROL.editorActionIconSize },
            }}
          >
            {actions}
          </Box>

          <EditorToolbarSeparator />
        </>
      ) : null}
    </div>
  );
}
