import { Alert, Checkbox, FormControlLabel, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { PackerDirectoryChips } from "@/applications/archives-packer/components/packing/PackerDirectoryChips";
import { PackerPathText } from "@/applications/archives-packer/components/packing/PackerPathText";
import { PackerSummaryRow } from "@/applications/archives-packer/components/packing/PackerSummaryRow";
import {
  ARCHIVE_PACK_MODE,
  ARCHIVE_VOLUME_EXTENSION,
  ARCHIVE_VOLUME_SUFFIX,
  HEADER_ENTRY_POINT,
  isWholeDirectory,
  readHeaderValue,
} from "@/applications/archives-packer/lib/pack-config";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { bytesToWholeMegabytes } from "@/lib/memory/size";
import { Nullable } from "@/lib/types/general";

interface IPackerConfirmSummaryProps {
  config: ArchivePackConfig;
  /** Volumes of this set the destination already holds, which packing refuses to replace unasked. */
  publishedVolumes: Array<string>;
  isForced: boolean;
  onForceChange: (isForced: boolean) => void;
}

/**
 * What a pack run is about to do, read from the configuration it will be given.
 *
 * Shown before packing because the selection rules are easy to get wrong in ways that only surface as a missing file
 * in game, and because a destination already holding this set is where the run turns destructive: replacing it is
 * asked for here rather than assumed.
 */
export function PackerConfirmSummary({
  config,
  publishedVolumes,
  isForced,
  onForceChange,
}: IPackerConfirmSummaryProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);
  const volumeName: string = `${config.name}.${ARCHIVE_VOLUME_SUFFIX[config.volumeExtension]}`;

  return (
    <Stack spacing={1.5}>
      <PackerSummaryRow label={"Source"}>
        <PackerPathText value={config.source} />
      </PackerSummaryRow>

      <PackerSummaryRow label={"Output"}>
        <PackerPathText value={config.destination} />
      </PackerSummaryRow>

      <PackerSummaryRow label={"Volumes"}>
        <Typography variant={"body2"}>
          <span className={"monospace"}>{volumeName}</span>, splitting every{" "}
          {bytesToWholeMegabytes(config.maxVolumeSize)} MB
        </Typography>
      </PackerSummaryRow>

      <PackerSummaryRow label={"Contents"}>
        {isWholeDirectory(config) ? (
          <Typography variant={"body2"}>Everything under the source directory</Typography>
        ) : (
          <Stack spacing={0.5}>
            {config.includeDirectories.length ? (
              <PackerDirectoryChips directories={config.includeDirectories} recursiveSuffix={" and below"} />
            ) : null}
            {config.includeFiles.length ? (
              <Typography variant={"body2"}>{config.includeFiles.join(", ")}</Typography>
            ) : null}
          </Stack>
        )}
      </PackerSummaryRow>

      {config.excludeDirectories.length || config.excludeExtensions.length ? (
        <PackerSummaryRow label={"Excluding"}>
          <Stack spacing={0.5}>
            {config.excludeDirectories.length ? (
              <PackerDirectoryChips directories={config.excludeDirectories} recursiveSuffix={" and below"} />
            ) : null}
            {config.excludeExtensions.length ? (
              <Typography variant={"body2"}>{config.excludeExtensions.join(", ")}</Typography>
            ) : null}
          </Stack>
        </PackerSummaryRow>
      ) : null}

      <PackerSummaryRow label={"Compression"}>
        <Typography variant={"body2"}>
          {config.mode === ARCHIVE_PACK_MODE.Store
            ? "Stored, nothing compressed"
            : "Compressed where the engine expects it"}
          {config.isWithSkipList ? ", editor leftovers skipped" : ", keeping editor leftovers"}
        </Typography>
      </PackerSummaryRow>

      <PackerSummaryRow label={"Mounts at"}>
        {entryPoint ? (
          <Typography variant={"body2"} className={"monospace"}>
            {entryPoint}
          </Typography>
        ) : (
          <Typography variant={"body2"} color={"text.secondary"}>
            Not set
          </Typography>
        )}
      </PackerSummaryRow>

      {publishedVolumes.length ? (
        <Alert severity={"warning"}>
          <Stack spacing={0.5}>
            <Typography variant={"body2"}>
              The output directory already holds {publishedVolumes.length} volume(s) of {volumeName}. Packing writes
              over them and cannot put them back if it stops partway.
            </Typography>

            <FormControlLabel
              control={
                <Checkbox size={"small"} checked={isForced} onChange={(event) => onForceChange(event.target.checked)} />
              }
              label={<Typography variant={"body2"}>Replace them</Typography>}
            />
          </Stack>
        </Alert>
      ) : null}

      {entryPoint || config.volumeExtension === ARCHIVE_VOLUME_EXTENSION.Xdb ? null : (
        <Alert severity={"error"}>
          Without an entry point the engine reads these as encrypted Shadow of Chernobyl archives.
        </Alert>
      )}
    </Stack>
  );
}
