import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveParticlesGroup, ArchiveParticlesGroupEffect } from "@/core/ipc/types/xrf-app";
import { MONOSPACE, PANEL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { describeGroupEffectChildren, describeGroupEffectName } from "./ArchiveParticlesDescriptionView.utils";

interface IArchiveParticlesGroupRowProps extends BaseComponentProps {
  group: ArchiveParticlesGroup;
}

/**
 * One sequence: the effects it plays, in the slots it declares them.
 */
export function ArchiveParticlesGroupRow({
  "data-testid": dataTestId = "archive-particles-group-row",
  id,
  className,
  group,
}: IArchiveParticlesGroupRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ minWidth: 0, paddingY: PANEL.propertyPaddingY, lineHeight: PANEL.contentLineHeight }}
    >
      <Box sx={{ display: "flex", gap: 1, justifyContent: "space-between", minWidth: 0 }}>
        <Typography variant={"body2"} sx={{ ...MONOSPACE, minWidth: 0, overflowWrap: "anywhere" }}>
          {group.name}
        </Typography>

        <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0, whiteSpace: "nowrap" }}>
          {`${group.effects.length} ${group.effects.length === 1 ? "effect" : "effects"}`}
        </Typography>
      </Box>

      {group.effects.map((slot: ArchiveParticlesGroupEffect, index: number) => (
        <Typography
          key={`${index}-${slot.effect.name}`}
          variant={"caption"}
          sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}
        >
          {[describeGroupEffectName(slot.effect), ...describeGroupEffectChildren(slot)].join(" · ")}
        </Typography>
      ))}
    </Box>
  );
}
