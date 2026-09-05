import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { IMAGE_CHECKERBOARD } from "@/core/ui/media/media.styles";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ITexturePreviewFrameProps extends BaseComponentProps {
  /** What the picture is, or what is happening to it, on the bar beneath. */
  caption: string;
  /** Whether the content area draws the alpha checkerboard behind whatever it holds. */
  isCheckered?: boolean;
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
        sx={[
          { display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 },
          isCheckered ? IMAGE_CHECKERBOARD : {},
        ]}
      >
        {children}
      </Box>

      <Box sx={{ flexShrink: 0, paddingX: 1.5, paddingY: 0.5, borderTop: 1, borderColor: "divider" }}>
        <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
          {caption}
        </Typography>
      </Box>
    </Box>
  );
}
