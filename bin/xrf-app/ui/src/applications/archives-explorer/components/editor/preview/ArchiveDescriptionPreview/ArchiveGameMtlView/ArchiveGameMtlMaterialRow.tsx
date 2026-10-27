import { ArchiveGameMtlMaterial } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

import { describeFactors, describeMaterial, describePhysics } from "./ArchiveGameMtlView.utils";

interface IArchiveGameMtlMaterialRowProps extends BaseComponentProps {
  material: ArchiveGameMtlMaterial;
}

/**
 * One game material: what it is called, what it does to what meets it, and what qualifies it.
 */
export function ArchiveGameMtlMaterialRow({
  "data-testid": dataTestId = "archive-game-mtl-material-row",
  id,
  className,
  material,
}: IArchiveGameMtlMaterialRowProps): ReactElement {
  const qualifier: Nullable<string> = describeMaterial(material);

  return (
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {material.name}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {describePhysics(material)}
        </Typography>
      </div>

      <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
        {describeFactors(material)}
      </Typography>

      {qualifier ? (
        <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
          {qualifier}
        </Typography>
      ) : null}
    </div>
  );
}
