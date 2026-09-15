import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { LAYOUT, MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveDescriptionRowProps extends BaseComponentProps {
  label: string;
  value: ReactNode;
  /** What qualifies the value: where a number came from, what a flag switches on. */
  caption?: ReactNode;
  /** Uses the shared identifier and path typography for the value. */
  isMonospace?: boolean;
}

/**
 * One labelled fact of a description.
 */
export function ArchiveDescriptionRow({
  "data-testid": dataTestId = "archive-description-row",
  id,
  className,
  label,
  value,
  caption,
  isMonospace = false,
}: IArchiveDescriptionRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      component={"dl"}
      sx={{
        display: "grid",
        gridTemplateColumns: { xs: "1fr", sm: `${LAYOUT.readingLabelWidth}px 1fr` },
        gap: PANEL.propertyValueGap,
        margin: 0,
        paddingY: PANEL.propertyPaddingY,
        minWidth: 0,
        lineHeight: PANEL.contentLineHeight,
      }}
    >
      <Typography
        component={"dt"}
        variant={"body2"}
        sx={{ color: "text.secondary", minWidth: 0, overflowWrap: "anywhere", lineHeight: "inherit" }}
      >
        {label}
      </Typography>

      <Box component={"dd"} sx={{ margin: 0, minWidth: 0 }}>
        <Typography
          component={"div"}
          variant={"body2"}
          sx={{
            ...(isMonospace ? MONOSPACE : null),
            lineHeight: "inherit",
            minWidth: 0,
            overflowWrap: "anywhere",
          }}
        >
          {value}
        </Typography>

        {caption ? (
          <Typography
            component={"div"}
            variant={"caption"}
            sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}
          >
            {caption}
          </Typography>
        ) : null}
      </Box>
    </Box>
  );
}
