import { bytesToWholeMegabytes } from "@xrf/format";
import { ArchivePackConfig, EArchivePackMode, EArchiveVolumeExtension } from "@xrf/ipc/types/xrf-pack";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Alert } from "@xrf/ui/feedback";
import { Checkbox, FormControlLabel } from "@xrf/ui/form/control";
import { Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { HEADER_ENTRY_POINT, readHeaderValue } from "@/core/archive/lib/header";
import { ARCHIVE_VOLUME_SUFFIX } from "@/core/archive/lib/volume-options";
import { isWholeDirectory } from "@/tools/archives-packer/lib/pack-config";

import { PackerDirectoryChips } from "./PackerDirectoryChips";

interface IPackerConfirmSummaryProps extends BaseComponentProps {
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
  "data-testid": dataTestId = "packer-confirm-summary",
  id,
  className,
  config,
  publishedVolumes,
  isForced,
  onForceChange,
}: IPackerConfirmSummaryProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);
  const volumeName: string = `${config.name}.${ARCHIVE_VOLUME_SUFFIX[config.volumeExtension]}`;

  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={1.5}>
      <EditorPanelProperty label={"Source"} value={config.source} isMonospace isInline />

      <EditorPanelProperty label={"Output"} value={config.destination} isMonospace isInline />

      <EditorPanelProperty
        label={"Volumes"}
        value={
          <>
            <span className={"monospace"}>{volumeName}</span>, splitting every{" "}
            {bytesToWholeMegabytes(config.maxVolumeSize)} MB
          </>
        }
        isInline
      />

      <EditorPanelProperty
        label={"Contents"}
        value={
          isWholeDirectory(config) ? (
            "Everything under the source directory"
          ) : (
            <Stack spacing={0.5}>
              {config.includeDirectories.length ? (
                <PackerDirectoryChips directories={config.includeDirectories} recursiveSuffix={" and below"} />
              ) : null}
              {config.includeFiles.length ? <span>{config.includeFiles.join(", ")}</span> : null}
            </Stack>
          )
        }
        isInline
      />

      {config.excludeDirectories.length || config.excludeExtensions.length ? (
        <EditorPanelProperty
          label={"Excluding"}
          value={
            <Stack spacing={0.5}>
              {config.excludeDirectories.length ? (
                <PackerDirectoryChips directories={config.excludeDirectories} recursiveSuffix={" and below"} />
              ) : null}
              {config.excludeExtensions.length ? <span>{config.excludeExtensions.join(", ")}</span> : null}
            </Stack>
          }
          isInline
        />
      ) : null}

      <EditorPanelProperty
        label={"Compression"}
        value={
          (config.mode === EArchivePackMode.STORE
            ? "Stored, nothing compressed"
            : "Compressed where the engine expects it") +
          (config.isWithSkipList ? ", editor leftovers skipped" : ", keeping editor leftovers")
        }
        isInline
      />

      <EditorPanelProperty
        label={"Mounts at"}
        value={entryPoint ?? <span className={"text-text-secondary"}>Not set</span>}
        isMonospace={entryPoint !== null}
        isInline
      />

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

      {entryPoint || config.volumeExtension === EArchiveVolumeExtension.XDB ? null : (
        <Alert severity={"error"}>
          Without an entry point the engine reads these as encrypted Shadow of Chernobyl archives.
        </Alert>
      )}
    </Stack>
  );
}
