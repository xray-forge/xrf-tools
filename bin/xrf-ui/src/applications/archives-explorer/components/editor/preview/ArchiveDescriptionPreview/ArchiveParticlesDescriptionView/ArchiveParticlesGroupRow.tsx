import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveParticlesGroup, ArchiveParticlesGroupEffect } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
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
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {group.name}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {`${group.effects.length} ${group.effects.length === 1 ? "effect" : "effects"}`}
        </Typography>
      </div>

      {group.effects.map((slot: ArchiveParticlesGroupEffect, index: number) => (
        <Typography
          key={`${index}-${slot.effect.name}`}
          className={"block wrap-anywhere text-text-disabled"}
          variant={"caption"}
        >
          {[describeGroupEffectName(slot.effect), ...describeGroupEffectChildren(slot)].join(" · ")}
        </Typography>
      ))}
    </div>
  );
}
