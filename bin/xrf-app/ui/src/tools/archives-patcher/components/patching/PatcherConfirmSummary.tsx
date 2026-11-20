import { EMPTY_ARRAY } from "@xrf/collections";
import { bytesToWholeMegabytes } from "@xrf/format";
import { ArchivePatchConfig, EArchivePackMode } from "@xrf/ipc/types/xrf-pack";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Alert } from "@xrf/ui/feedback";
import { ReactElement } from "react";

import { ArchiveRunConfirmSummary } from "@/core/archive/components/config-editor/ArchiveRunConfirmSummary";
import { HEADER_ENTRY_POINT, readHeaderValue } from "@/core/archive/lib/header";
import { ARCHIVE_VOLUME_SUFFIX } from "@/core/archive/lib/volume-options";

interface IPatcherConfirmSummaryProps extends BaseComponentProps {
  config: ArchivePatchConfig;
  /** Whether the run reports the difference or writes it, which changes almost everything below. */
  isPreviewOnly: boolean;
  /** Volumes of this set the output already holds, which publishing refuses to replace unasked. */
  publishedVolumes: ReadonlyArray<string>;
  isForced: boolean;
  onForceChange: (isForced: boolean) => void;
}

/**
 * What a run is about to do, read from the configuration it will be given.
 */
export function PatcherConfirmSummary({
  "data-testid": dataTestId = "patcher-confirm-summary",
  id,
  className,
  config,
  isPreviewOnly,
  publishedVolumes,
  isForced,
  onForceChange,
}: IPatcherConfirmSummaryProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);
  const volumeName: string = `${config.name}.${ARCHIVE_VOLUME_SUFFIX[config.volumeExtension]}`;

  return (
    <ArchiveRunConfirmSummary
      data-testid={dataTestId}
      id={id}
      className={className}
      publishedVolumes={isPreviewOnly ? EMPTY_ARRAY : publishedVolumes}
      replaceWarning={
        `The output already holds ${publishedVolumes.length} volume(s) of this set. ` +
        "Replacing them cannot be undone if the run fails partway through."
      }
      isForced={isForced}
      onForceChange={onForceChange}
    >
      <EditorPanelProperty label={"Game"} value={config.input} isMonospace isInline />

      <EditorPanelProperty
        label={"Delivers"}
        value={config.target ?? "Its own loose gamedata"}
        isMonospace={config.target !== null}
        isInline
      />

      <EditorPanelProperty
        label={"Compares"}
        value={
          (config.include.length ? config.include.join(", ") : "Everything both sides hold") +
          (config.ignore.length ? `, except ${config.ignore.join(", ")}` : "") +
          (config.excludeExtensions.length ? `, excluding ${config.excludeExtensions.join(", ")}` : "")
        }
        isInline
      />

      {isPreviewOnly ? (
        <Alert severity={"info"} variant={"outlined"}>
          Reports what differs and writes nothing.
        </Alert>
      ) : (
        <>
          <EditorPanelProperty label={"Writes"} value={`${config.destination}\\${volumeName}`} isMonospace isInline />

          <EditorPanelProperty
            label={"Mounts at"}
            value={entryPoint ?? "nothing — the engine will read these as encrypted archives"}
            isMonospace
            isInline
          />

          <EditorPanelProperty
            label={"Volumes"}
            value={
              `${config.mode === EArchivePackMode.STORE ? "Stored" : "Compressed"}, up to ` +
              `${bytesToWholeMegabytes(config.maxVolumeSize)} MB each`
            }
            isInline
          />
        </>
      )}
    </ArchiveRunConfirmSummary>
  );
}
