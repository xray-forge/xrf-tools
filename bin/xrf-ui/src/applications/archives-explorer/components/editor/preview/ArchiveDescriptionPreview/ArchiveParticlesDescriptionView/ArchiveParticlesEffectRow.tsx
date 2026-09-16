import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveParticlesEffect } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
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
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {effect.name}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {describeEffectCounts(effect)}
        </Typography>
      </div>

      <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
        <ArchiveDescriptionReferenceLink reference={effect.texture} />
        {` · drawn with ${effect.shader}`}
        {status ? ` · ${status}` : null}
      </Typography>

      <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
        {describeEffectDetail(effect)}
      </Typography>
    </div>
  );
}
