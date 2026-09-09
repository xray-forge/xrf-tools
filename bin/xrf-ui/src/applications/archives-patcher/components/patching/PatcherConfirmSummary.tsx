import { Alert, Checkbox, FormControlLabel, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ARCHIVE_PACK_MODE, ARCHIVE_VOLUME_SUFFIX } from "@/applications/archives-packer/lib/pack-config";
import { ArchivePathText, ArchiveSummaryRow, HEADER_ENTRY_POINT, readHeaderValue } from "@/core/archive";
import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { bytesToWholeMegabytes } from "@/lib/memory/size";
import { Nullable } from "@/lib/types/general";

interface IPatcherConfirmSummaryProps {
  config: ArchivePatchConfig;
  /** Whether the run reports the difference or writes it, which changes almost everything below. */
  isPreviewOnly: boolean;
  /** Volumes of this set the output already holds, which publishing refuses to replace unasked. */
  publishedVolumes: Array<string>;
  isForced: boolean;
  onForceChange: (isForced: boolean) => void;
}

/**
 * What a run is about to do, read from the configuration it will be given.
 */
export function PatcherConfirmSummary({
  config,
  isPreviewOnly,
  publishedVolumes,
  isForced,
  onForceChange,
}: IPatcherConfirmSummaryProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);
  const volumeName: string = `${config.name}.${ARCHIVE_VOLUME_SUFFIX[config.volumeExtension]}`;

  return (
    <Stack spacing={1.5}>
      <ArchiveSummaryRow label={"Game"}>
        <ArchivePathText value={config.input} />
      </ArchiveSummaryRow>

      <ArchiveSummaryRow label={"Delivers"}>
        {config.target === null ? (
          <Typography variant={"body2"}>Its own loose gamedata</Typography>
        ) : (
          <ArchivePathText value={config.target} />
        )}
      </ArchiveSummaryRow>

      <ArchiveSummaryRow label={"Compares"}>
        <Typography variant={"body2"}>
          {config.include.length ? config.include.join(", ") : "Everything both sides hold"}
          {config.ignore.length ? `, except ${config.ignore.join(", ")}` : ""}
          {config.excludeExtensions.length ? `, excluding ${config.excludeExtensions.join(", ")}` : ""}
        </Typography>
      </ArchiveSummaryRow>

      {isPreviewOnly ? (
        <Alert severity={"info"} variant={"outlined"}>
          Reports what differs and writes nothing.
        </Alert>
      ) : (
        <>
          <ArchiveSummaryRow label={"Writes"}>
            <ArchivePathText value={`${config.destination}\\${volumeName}`} />
          </ArchiveSummaryRow>

          <ArchiveSummaryRow label={"Mounts at"}>
            <Typography variant={"body2"} className={"monospace"}>
              {entryPoint ?? "nothing — the engine will read these as encrypted archives"}
            </Typography>
          </ArchiveSummaryRow>

          <ArchiveSummaryRow label={"Volumes"}>
            <Typography variant={"body2"}>
              {config.mode === ARCHIVE_PACK_MODE.Store ? "Stored" : "Compressed"}, up to{" "}
              {bytesToWholeMegabytes(config.maxVolumeSize)} MB each
            </Typography>
          </ArchiveSummaryRow>
        </>
      )}

      {!isPreviewOnly && publishedVolumes.length ? (
        <>
          <Alert severity={"warning"}>
            The output already holds {publishedVolumes.length} volume(s) of this set. Replacing them cannot be undone if
            the run fails partway through.
          </Alert>

          <FormControlLabel
            control={
              <Checkbox size={"small"} checked={isForced} onChange={(event) => onForceChange(event.target.checked)} />
            }
            label={<Typography variant={"body2"}>Replace them</Typography>}
          />
        </>
      ) : null}
    </Stack>
  );
}
