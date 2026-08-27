import { Box, Divider, Typography } from "@mui/material";
import { ReactElement } from "react";

import { DialogElementDescriptor } from "@/core/bindings/types/xrf-dialog";

export interface IDialogInspectorSectionProps {
  title: string;
  caption: string;
  elements: ReadonlyArray<DialogElementDescriptor>;
  /** Suppresses the leading divider, so the first group does not draw one against the band above it. */
  isFirst?: boolean;
}

/**
 * One titled group of a node's elements.
 */
export function DialogInspectorSection({
  title,
  caption,
  elements,
  isFirst,
}: IDialogInspectorSectionProps): ReactElement {
  return (
    <Box sx={{ paddingBottom: 1.5, paddingTop: isFirst ? 0 : 1.5, paddingX: 2 }}>
      {isFirst ? null : <Divider sx={{ marginBottom: 1.5, marginX: -2 }} />}

      <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
        {title}
      </Typography>

      <Typography variant={"caption"} sx={{ color: "text.disabled", display: "block" }}>
        {caption}
      </Typography>

      <Box sx={{ marginTop: 1 }}>
        {elements.map((element: DialogElementDescriptor, index: number) => (
          <Box
            key={`${element.name}-${index}`}
            sx={{ alignItems: "baseline", display: "flex", gap: 1.5, lineHeight: 1.6, paddingY: 0.4 }}
          >
            <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0 }}>
              {element.name}
            </Typography>

            <Typography
              variant={"body2"}
              sx={{ fontFamily: "monospace", marginLeft: "auto", overflowWrap: "anywhere", textAlign: "right" }}
            >
              {element.value}
            </Typography>
          </Box>
        ))}
      </Box>
    </Box>
  );
}
