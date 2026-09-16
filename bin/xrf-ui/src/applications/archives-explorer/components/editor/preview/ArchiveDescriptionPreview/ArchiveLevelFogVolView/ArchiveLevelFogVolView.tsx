import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveLevelFogVolDescription, ArchiveLevelFogVolume } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveLevelFogVolumeRow } from "./ArchiveLevelFogVolumeRow";

interface IArchiveLevelFogVolViewProps extends BaseComponentProps {
  description: ArchiveLevelFogVolDescription;
  scope: ArchiveDescribeScope;
}

/**
 * A level's volumetric fog: the bodies it simulates, and the geometry each one flows around.
 */
export function ArchiveLevelFogVolView({
  "data-testid": dataTestId = "archive-level-fog-vol-view",
  id,
  className,
  description,
  scope,
}: IArchiveLevelFogVolViewProps): ReactElement {
  const { volumes } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Volumetric fog"} isFirst>
        <ArchiveDescriptionRow
          label={"Bodies"}
          value={`${volumes.length}`}
          caption={volumes.length ? null : "The file was written, and nothing was placed in it"}
        />

        <ArchiveDescriptionRow
          label={"Obstacles"}
          value={formatCount(description.obstacles)}
          caption={"Geometry the simulation flows around, taken over every body"}
        />

        <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />
      </EditorPanelSection>

      {volumes.length ? (
        <EditorPanelSection
          title={`Bodies (${volumes.length})`}
          caption={"A body's own settings live in the LTX it names, not in this file"}
        >
          {volumes.map((volume: ArchiveLevelFogVolume, index: number) => (
            <ArchiveLevelFogVolumeRow key={index} volume={volume} index={index} scope={scope} />
          ))}
        </EditorPanelSection>
      ) : null}
    </ArchiveDescriptionLayout>
  );
}
