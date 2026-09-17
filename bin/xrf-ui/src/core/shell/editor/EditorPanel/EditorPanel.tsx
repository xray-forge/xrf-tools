import { Box } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { EditorPanelHeader } from "@/core/shell/editor/EditorPanelHeader";
import { mergeSx } from "@/core/theme/merge-sx";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelProps extends StyledComponentProps {
  /** The name this panel's stripe button carries. */
  title: string;
  /** Controls acting on the whole panel, at the end of the title row. */
  actions?: ReactNode;
  children: ReactNode;
}

/**
 * One panel, titled with the name its stripe button carries.
 */
export function EditorPanel({
  "data-testid": dataTestId = "editor-panel",
  id,
  className,
  sx,
  title,
  actions,
  children,
}: IEditorPanelProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={mergeSx({ display: "flex", flexDirection: "column", minWidth: 0, minHeight: 0 }, sx)}
    >
      <EditorPanelHeader title={title} actions={actions} />

      <div className={"min-h-0 min-w-0 grow"}>{children}</div>
    </Box>
  );
}
