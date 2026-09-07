import { Box, Divider, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelSectionProps extends BaseComponentProps {
  title: ReactNode;
  /** What distinguishes this group from a similar one beside it. */
  caption?: ReactNode;
  children: ReactNode;
  /** Suppresses the leading divider, so the first group does not draw one against the panel title. */
  isFirst?: boolean;
  /**
   * Takes the height the panel has left over, for content that scrolls on its own rather than flowing.
   */
  isFilling?: boolean;
}

/**
 * A titled group of rows.
 */
export function EditorPanelSection({
  "data-testid": dataTestId = "editor-panel-section",
  id,
  className,
  title,
  caption,
  children,
  isFirst,
  isFilling,
}: IEditorPanelSectionProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        minWidth: 0,
        paddingX: PANEL.contentPadding,
        paddingTop: isFirst ? PANEL.contentPadding : PANEL.sectionPaddingY,
        paddingBottom: PANEL.sectionPaddingY,
        ...(isFilling ? { display: "flex", flexDirection: "column", flexGrow: 1, minHeight: 0 } : null),
      }}
    >
      {isFirst ? null : <Divider sx={{ marginBottom: PANEL.sectionPaddingY, marginX: -PANEL.contentPadding }} />}

      <Typography component={"h3"} variant={"overline"} sx={{ color: "text.secondary", overflowWrap: "anywhere" }}>
        {title}
      </Typography>

      {caption ? (
        <Typography variant={"caption"} sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}>
          {caption}
        </Typography>
      ) : null}

      <Box
        sx={{ marginTop: PANEL.sectionContentGap, minWidth: 0, ...(isFilling ? { flexGrow: 1, minHeight: 0 } : null) }}
      >
        {children}
      </Box>
    </Box>
  );
}
