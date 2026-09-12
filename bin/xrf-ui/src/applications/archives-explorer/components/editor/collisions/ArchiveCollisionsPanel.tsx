import { Box, Divider, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { XrayPathCollision } from "@/core/bindings/types/xrf-vfs";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveCollisionRow } from "./ArchiveCollisionRow";

export interface IArchiveCollisionsPanelProps extends BaseComponentProps {
  archivesService: ArchivesService;
}

/**
 * Every entry the open volume set holds that no engine lookup can reach.
 *
 * todo: Inject or supply data directly as props.
 */
export function ArchiveCollisionsPanel({ archivesService }: IArchiveCollisionsPanelProps): ReactElement {
  const collisions: Array<XrayPathCollision> = archivesService.collisions.value ?? [];

  return (
    <Box sx={{ display: "flex", flexDirection: "column", minHeight: 0 }}>
      <Box sx={{ padding: 2 }}>
        <Typography variant={"subtitle2"}>Unreachable files</Typography>
        <Typography variant={"caption"} sx={{ display: "block", marginTop: 0.5, color: "text.secondary" }}>
          {archivesService.collisions.error
            ? "Could not read what this volume set cannot reach."
            : collisions.length
              ? `${collisions.length} entry(ies) fold onto a path another entry already claims.`
              : "Every entry in this volume set resolves to a path of its own. No collissions."}
        </Typography>
      </Box>

      {collisions.length ? <Divider /> : null}

      <Box sx={{ minHeight: 0, overflowY: "auto" }}>
        {collisions.map((collision: XrayPathCollision) => (
          <ArchiveCollisionRow key={`${collision.logicalPath}:${collision.unreachable}`} collision={collision} />
        ))}
      </Box>
    </Box>
  );
}
