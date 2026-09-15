import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveParticlesEffect } from "@/core/ipc/types/xrf-app";
import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeReferenceStatus } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";
import { describeEffectCounts, describeEffectDetail } from "./ArchiveParticlesDescriptionView.utils";

interface IArchiveParticlesEffectRowProps extends BaseComponentProps {
  effect: ArchiveParticlesEffect;
  scope: ArchiveDescribeScope;
}

/**
 * One emitter: what it draws with, how much of it, and what moves it.
 */
export function ArchiveParticlesEffectRow({
  "data-testid": dataTestId = "archive-particles-effect-row",
  id,
  className,
  effect,
  scope,
}: IArchiveParticlesEffectRowProps): ReactElement {
  const status: Nullable<string> = describeReferenceStatus(effect.texture, scope);

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ minWidth: 0, paddingY: PANEL.propertyPaddingY, lineHeight: PANEL.contentLineHeight }}
    >
      <Box sx={{ display: "flex", gap: 1, justifyContent: "space-between", minWidth: 0 }}>
        <Typography variant={"body2"} sx={{ ...MONOSPACE, minWidth: 0, overflowWrap: "anywhere" }}>
          {effect.name}
        </Typography>

        <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0, whiteSpace: "nowrap" }}>
          {describeEffectCounts(effect)}
        </Typography>
      </Box>

      <Typography variant={"caption"} sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}>
        <ArchiveDescriptionReferenceLink reference={effect.texture} />
        {` · drawn with ${effect.shader}`}
        {status ? ` · ${status}` : null}
      </Typography>

      <Typography variant={"caption"} sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}>
        {describeEffectDetail(effect)}
      </Typography>
    </Box>
  );
}
