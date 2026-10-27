import { formatCount, formatDuration } from "@xrf/format";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { EquipmentSpritePackResult } from "@xrf/ipc/types/xrf-texture";
import { ReactElement } from "react";

import { CommandResult } from "@/core/ui/command-result/CommandResult";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEquipmentPackResultProps extends BaseComponentProps {
  result: EquipmentSpritePackResult;
}

/** What a pack wrote: the icons drawn, the ones without an image, and the sheet's size. */
export function EquipmentPackResult({
  "data-testid": dataTestId = "equipment-pack-result",
  id,
  className,
  result,
}: IEquipmentPackResultProps): ReactElement {
  // The sheet is written once at the end, so a stopped pack wrote nothing, whatever it had drawn.
  if (result.outcome === EJobOutcome.CANCELLED) {
    return (
      <CommandResult
        data-testid={dataTestId}
        id={id}
        className={className}
        headline={"Stopped before writing. The sprite was left unchanged."}
        tone={"info"}
        stats={[{ label: "elapsed", value: formatDuration(result.duration) }]}
      />
    );
  }

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={
        result.skippedCount
          ? `Packed ${formatCount(result.packedCount)} icon(s), ${formatCount(result.skippedCount)} without an image`
          : `Packed ${formatCount(result.packedCount)} icon(s)`
      }
      tone={result.skippedCount ? "warning" : "success"}
      stats={[
        { label: "packed", value: result.packedCount },
        { label: "skipped", value: result.skippedCount, tone: result.skippedCount ? "warning" : undefined },
        { label: "sprite", value: `${result.savedWidth} × ${result.savedHeight}` },
        { label: "elapsed", value: formatDuration(result.duration) },
      ]}
    />
  );
}
