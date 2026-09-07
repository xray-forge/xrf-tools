import { Box, Typography } from "@mui/material";
import { Fragment, ReactElement, ReactNode } from "react";

import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { splitAfterSeparators } from "@/lib/path/separator";

/**
 * A path rendered so it breaks after a separator rather than mid-name.
 *
 * `overflow-wrap` alone breaks wherever the line runs out, which turns `act\actor\act_stalker` into a name split
 * across two lines at no meaningful point. Offering the separators as break opportunities first means a path folds
 * where a reader would fold it; the wrap rule stays behind it for a single name longer than the panel.
 */
function toBreakableSegments(value: string): ReactNode {
  const segments: Array<string> = splitAfterSeparators(value);

  if (segments.length < 2) {
    return value;
  }

  return segments.map((segment: string, index: number) => (
    <Fragment key={index}>
      {segment}
      <wbr />
    </Fragment>
  ));
}

interface IEditorPanelPropertyProps extends BaseComponentProps {
  label: string;
  value: ReactNode;
  /** Uses the shared identifier and path typography for the value. */
  isMonospace?: boolean;
}

/** Displays a muted label above its full-width value, preserving space for paths and descriptions in narrow panels. */
export function EditorPanelProperty({
  "data-testid": dataTestId = "editor-panel-property",
  id,
  className,
  label,
  value,
  isMonospace = false,
}: IEditorPanelPropertyProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      component={"dl"}
      sx={{
        display: "flex",
        flexDirection: "column",
        gap: PANEL.propertyValueGap,
        margin: 0,
        paddingY: PANEL.propertyPaddingY,
        minWidth: 0,
        lineHeight: PANEL.contentLineHeight,
      }}
    >
      <Typography
        component={"dt"}
        variant={"caption"}
        sx={{
          color: "text.secondary",
          maxWidth: "100%",
          overflowWrap: "anywhere",
          lineHeight: "inherit",
        }}
      >
        {label}
      </Typography>

      <Typography
        component={"dd"}
        variant={"body2"}
        sx={{
          ...(isMonospace ? MONOSPACE : null),
          margin: 0,
          lineHeight: "inherit",
          whiteSpace: "pre-wrap",
          minWidth: 0,
          width: "100%",
          maxWidth: "100%",
          overflowWrap: "anywhere",
          textAlign: "start",
        }}
      >
        {isMonospace && typeof value === "string" ? toBreakableSegments(value) : value}
      </Typography>
    </Box>
  );
}
