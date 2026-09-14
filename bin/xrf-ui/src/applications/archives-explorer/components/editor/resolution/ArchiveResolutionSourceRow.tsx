import { default as FolderIcon } from "@mui/icons-material/FolderOutlined";
import { default as ArchiveIcon } from "@mui/icons-material/Inventory2Outlined";
import { Box, Chip, chipClasses, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveResolutionSource, ArchiveResolutionVolume } from "@/core/ipc/types/xrf-app";
import { EXraySourceKind } from "@/core/ipc/types/xrf-vfs";
import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveResolutionVolumeRow } from "./ArchiveResolutionVolumeRow";

export interface IArchiveResolutionSourceRowProps extends BaseComponentProps {
  source: ArchiveResolutionSource;
  /** Position of the source in search order, counted from one. */
  rank: number;
}

/**
 * One source of the search, at the position it is asked.
 */
export function ArchiveResolutionSourceRow({
  "data-testid": dataTestId = "archive-resolution-source-row",
  id,
  className,
  source,
  rank,
}: IArchiveResolutionSourceRowProps): ReactElement {
  const isLoose: boolean = source.kind === EXraySourceKind.DIRECTORY;

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", gap: 1.5, paddingY: 1, borderTop: 1, borderColor: "divider" }}
    >
      <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0, width: 24, textAlign: "right" }}>
        {rank}
      </Typography>

      <Box sx={{ display: "flex", flexDirection: "column", gap: 0.5, flexGrow: 1, minWidth: 0 }}>
        <Box sx={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 1 }}>
          <Chip
            size={"small"}
            variant={"outlined"}
            color={isLoose ? "success" : "default"}
            icon={isLoose ? <FolderIcon /> : <ArchiveIcon />}
            label={isLoose ? "Files" : "Archives"}
            sx={{ [`& .${chipClasses.icon}`]: { fontSize: 14 }, paddingX: 1, paddingY: 1 }}
          />

          {source.origin ? (
            <Typography variant={"body2"} sx={{ ...MONOSPACE, overflowWrap: "anywhere" }}>
              {source.origin}
            </Typography>
          ) : null}

          {source.base ? (
            <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
              {`mounted at ${source.base}`}
            </Typography>
          ) : null}
        </Box>

        <Typography variant={"body2"} sx={{ ...MONOSPACE, overflowWrap: "anywhere" }}>
          {source.path}
        </Typography>

        <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block" }}>
          {`${source.entries.toLocaleString()} entries · reached through ${source.step}`}
        </Typography>

        {source.volumes.length ? (
          <Box sx={{ marginTop: 0.5, paddingLeft: 1, borderLeft: 1, borderColor: "divider" }}>
            {source.volumes.map((volume: ArchiveResolutionVolume, index: number) => (
              <ArchiveResolutionVolumeRow key={volume.path} volume={volume} rank={index + 1} />
            ))}
          </Box>
        ) : null}
      </Box>
    </Box>
  );
}
