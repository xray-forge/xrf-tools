import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveLevelPsStaticEffect } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { formatCount } from "../ArchiveDescriptionPreview.utils";

interface IArchiveLevelPsStaticEffectRowProps extends BaseComponentProps {
  effect: ArchiveLevelPsStaticEffect;
}

/**
 * One effect the level plants, and how widely.
 */
export function ArchiveLevelPsStaticEffectRow({
  "data-testid": dataTestId = "archive-level-ps-static-effect-row",
  id,
  className,
  effect,
}: IArchiveLevelPsStaticEffectRowProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {effect.name}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {`${formatCount(effect.placements)} ${effect.placements === 1 ? "placement" : "placements"}`}
        </Typography>
      </div>

      {effect.restricted ? (
        <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
          {`${formatCount(effect.restricted)} of them only some multiplayer modes load`}
        </Typography>
      ) : null}
    </div>
  );
}
