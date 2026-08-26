import { Box } from "@mui/material";
import { ReactElement, ReactNode, useCallback } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { IApplicationDescriptor } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { EditorToolbarCrumb } from "@/core/shell/editor/EditorToolbarCrumb";
import { EditorToolbarPathSeparator } from "@/core/shell/editor/EditorToolbarPathSeparator";
import { useIsEditorBusy } from "@/core/shell/EditorBusyContext";
import { useRequestLeave } from "@/core/shell/EditorDirtyContext";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IEditorToolbarProps extends BaseComponentProps {
  /** Overrides the application name resolved from the route. Rarely needed. */
  title?: string;
  /** The open document, as the last breadcrumb segment. Counts and state belong in the status bar. */
  subtitle?: ReactNode;
  actions?: ReactNode;
  /**
   * Returns to the application's own level, which is the level with nothing open, so this closes the
   * open document. Omit it and the segment is inert.
   */
  onBack?: () => void;
}

/**
 * The active application's row inside the window caption.
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
  const application: Nullable<IApplicationDescriptor> = useCurrentApplication();
  const navigate: NavigateFunction = useNavigate();

  const isBusy: boolean = useIsEditorBusy();
  const requestLeave: (leave: () => void) => void = useRequestLeave();

  const label: Nullable<string> = title ?? application?.label ?? null;

  // Asks first when the editor is holding unsaved work; goes straight home when it is not.
  const onGoHome = useCallback(() => requestLeave(() => navigate("/", { replace: true })), [navigate, requestLeave]);

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", alignItems: "center", gap: 0.75, width: "100%", height: "100%", minWidth: 0 }}
    >
      <EditorToolbarCrumb label={"XRF"} isDisabled={isBusy} hint={"Back to all tools"} onClick={onGoHome} />

      {label ? (
        <>
          <EditorToolbarPathSeparator />

          <EditorToolbarCrumb
            label={label}
            isDisabled={isBusy}
            accessibleName={onBack ? `Back to ${label}` : undefined}
            hint={onBack ? `Back to ${label}, closing what is open` : undefined}
            onClick={onBack}
          />
        </>
      ) : null}

      {subtitle ? (
        <>
          <EditorToolbarPathSeparator />

          <Box
            sx={{
              direction: "rtl",
              textAlign: "left",
              minWidth: 0,
              overflow: "hidden",
              whiteSpace: "nowrap",
              textOverflow: "ellipsis",
              fontSize: "0.75rem",
              opacity: 0.7,
              "& > *": { direction: "ltr" },
            }}
          >
            {subtitle}
          </Box>
        </>
      ) : null}

      <Box sx={{ flexGrow: 1, minWidth: 8 }} />

      {actions ? (
        <>
          <Box
            sx={{
              display: "flex",
              alignItems: "center",
              flexShrink: 0,
              "& .MuiIconButton-root": { width: 24, height: 24, padding: 0 },
              "& .MuiSvgIcon-root": { fontSize: 16 },
            }}
          >
            {actions}
          </Box>

          <Box
            aria-hidden={true}
            sx={{ width: "1px", height: 18, marginLeft: 0.5, flexShrink: 0, backgroundColor: "divider" }}
          />
        </>
      ) : null}
    </Box>
  );
}
