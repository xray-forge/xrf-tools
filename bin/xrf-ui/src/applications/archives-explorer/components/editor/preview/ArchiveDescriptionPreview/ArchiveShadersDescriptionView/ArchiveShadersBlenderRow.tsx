import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveShadersBlender, ArchiveShadersProperty } from "@/core/ipc/types/xrf-app";
import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { describeReferenceStatus } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";

interface IArchiveShadersBlenderRowProps extends BaseComponentProps {
  blender: ArchiveShadersBlender;
  scope: ArchiveDescribeScope;
}

/**
 * One blender: the shader name, the class that decides its passes, and the grid an author left behind.
 */
export function ArchiveShadersBlenderRow({
  "data-testid": dataTestId = "archive-shaders-blender-row",
  id,
  className,
  blender,
  scope,
}: IArchiveShadersBlenderRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ minWidth: 0, paddingY: PANEL.propertyPaddingY, lineHeight: PANEL.contentLineHeight }}
    >
      <Box sx={{ display: "flex", gap: 1, justifyContent: "space-between", minWidth: 0 }}>
        <Typography variant={"body2"} sx={{ ...MONOSPACE, minWidth: 0, overflowWrap: "anywhere" }}>
          {blender.name}
        </Typography>

        <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0, whiteSpace: "nowrap" }}>
          {`${blender.class} · v${blender.version}`}
        </Typography>
      </Box>

      {blender.properties.map((property: ArchiveShadersProperty, index: number) => (
        <Typography
          key={`${index}-${property.name}`}
          variant={"caption"}
          sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}
        >
          {`${property.name}: `}

          {property.texture ? (
            <>
              <ArchiveDescriptionReferenceLink reference={property.texture} />
              {` — ${describeReferenceStatus(property.texture, scope) ?? ""}`}
            </>
          ) : (
            property.value || property.kind.toLowerCase()
          )}
        </Typography>
      ))}
    </Box>
  );
}
