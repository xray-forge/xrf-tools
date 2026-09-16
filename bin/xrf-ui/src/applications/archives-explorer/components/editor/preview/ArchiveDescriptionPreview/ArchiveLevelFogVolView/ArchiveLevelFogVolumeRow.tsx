import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveLevelFogVolume } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeReferenceStatus, formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";

interface IArchiveLevelFogVolumeRowProps extends BaseComponentProps {
  volume: ArchiveLevelFogVolume;
  /** Position in the file, which is the only name a body has. */
  index: number;
  scope: ArchiveDescribeScope;
}

/**
 * One fog body: the config its simulation is read from, and how much geometry that simulation flows around.
 */
export function ArchiveLevelFogVolumeRow({
  "data-testid": dataTestId = "archive-level-fog-volume-row",
  id,
  className,
  volume,
  index,
  scope,
}: IArchiveLevelFogVolumeRowProps): ReactElement {
  const status: Nullable<string> = volume.profile ? describeReferenceStatus(volume.profile, scope) : null;
  const obstacles: string = `${formatCount(volume.obstacles)} ${volume.obstacles === 1 ? "obstacle" : "obstacles"}`;

  return (
    <ArchiveDescriptionRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={`Body ${index + 1}`}
      isMonospace
      value={volume.profile ? <ArchiveDescriptionReferenceLink reference={volume.profile} /> : "Names no profile"}
      caption={status ? `${obstacles} · ${status}` : obstacles}
    />
  );
}
