import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { IMAGE_CHECKERBOARD } from "@/core/ui/media/media.styles";
import { tid, uid } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ITexturePreviewFrameProps extends BaseComponentProps {
  /** What the picture is, or what is happening to it, on the bar beneath. */
  caption: string;
  /** Whether the content area draws the alpha checkerboard behind whatever it holds. */
  isCheckered?: boolean;
  /** Controls sitting at the far end of the caption bar, which stay put while the content changes. */
  actions?: ReactNode;
  children: ReactNode;
}

/**
 * The shell the preview keeps whatever it is showing.
 */
export function TexturePreviewFrame({
  "data-testid": dataTestId = "texture-preview-frame",
  id,
  className,
  caption,
  isCheckered = true,
  actions,
  children,
}: ITexturePreviewFrameProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}
    >
      <Box
        data-testid={tid(dataTestId, "body")}
        id={uid(id, "body")}
        sx={[
          { display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 },
          isCheckered ? IMAGE_CHECKERBOARD : {},
        ]}
      >
        {children}
      </Box>

      <Box
        data-testid={tid(dataTestId, "footer")}
        id={uid(id, "footer")}
        sx={{
          alignItems: "center",
          borderColor: "divider",
          borderTop: 1,
          display: "flex",
          flexShrink: 0,
          gap: 1,
          justifyContent: "space-between",
          minHeight: 34,
          paddingX: 1.5,
          paddingY: 0.5,
        }}
      >
        <Typography variant={"caption"} noWrap sx={{ color: "text.secondary" }}>
          {caption}
        </Typography>

        {actions}
      </Box>
    </Box>
  );
}
