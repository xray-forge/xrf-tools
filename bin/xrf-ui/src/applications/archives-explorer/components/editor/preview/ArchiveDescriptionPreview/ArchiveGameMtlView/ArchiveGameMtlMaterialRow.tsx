import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveGameMtlMaterial } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
