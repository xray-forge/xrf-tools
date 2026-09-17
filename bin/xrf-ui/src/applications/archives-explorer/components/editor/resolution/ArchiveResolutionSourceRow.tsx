import { default as FolderIcon } from "@mui/icons-material/FolderOutlined";
import { default as ArchiveIcon } from "@mui/icons-material/Inventory2Outlined";
import { Chip, chipClasses, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveResolutionSource, ArchiveResolutionVolume } from "@/core/ipc/types/xrf-app";
import { EXraySourceKind } from "@/core/ipc/types/xrf-vfs";
import { cn } from "@/lib/dom/dom-name";
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
    <div data-testid={dataTestId} id={id} className={cn("flex gap-3 border-t border-divider py-2", className)}>
      <Typography className={"w-6 shrink-0 text-right text-text-secondary"} variant={"body2"}>
        {rank}
      </Typography>

      <div className={"flex min-w-0 grow flex-col gap-1"}>
        <div className={"flex flex-wrap items-center gap-2"}>
          <Chip
            className={"p-2"}
            size={"small"}
            variant={"outlined"}
            color={isLoose ? "success" : "default"}
            icon={isLoose ? <FolderIcon /> : <ArchiveIcon />}
            label={isLoose ? "Files" : "Archives"}
            sx={{ [`& .${chipClasses.icon}`]: { fontSize: 14 } }}
          />

          {source.origin ? (
            <Typography className={"monospace wrap-anywhere"} variant={"body2"}>
              {source.origin}
            </Typography>
          ) : null}

          {source.base ? (
            <Typography className={"text-text-secondary"} variant={"caption"}>
              {`mounted at ${source.base}`}
            </Typography>
          ) : null}
        </div>

        <Typography className={"monospace wrap-anywhere"} variant={"body2"}>
          {source.path}
        </Typography>

        <Typography className={"block text-text-secondary"} variant={"caption"}>
          {`${source.entries.toLocaleString()} entries · reached through ${source.step}`}
        </Typography>

        {source.volumes.length ? (
          <div className={"mt-1 border-l border-divider pl-2"}>
            {source.volumes.map((volume: ArchiveResolutionVolume, index: number) => (
              <ArchiveResolutionVolumeRow key={volume.path} volume={volume} rank={index + 1} />
            ))}
          </div>
        ) : null}
      </div>
    </div>
  );
}
