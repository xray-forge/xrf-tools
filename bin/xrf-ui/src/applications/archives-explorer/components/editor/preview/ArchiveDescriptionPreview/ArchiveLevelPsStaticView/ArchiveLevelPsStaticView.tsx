import { ReactElement } from "react";

import { ArchiveLevelPsStaticDescription, ArchiveLevelPsStaticEffect } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveLevelPsStaticEffectRow } from "./ArchiveLevelPsStaticEffectRow";

interface IArchiveLevelPsStaticViewProps extends BaseComponentProps {
  description: ArchiveLevelPsStaticDescription;
}

/**
 * The particle effects a level plants: what is planted, rather than where each copy of it stands.
 */
export function ArchiveLevelPsStaticView({
  "data-testid": dataTestId = "archive-level-ps-static-view",
  id,
  className,
  description,
}: IArchiveLevelPsStaticViewProps): ReactElement {
  const { effects } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Planted effects"} isFirst>
        <ArchiveDescriptionRow
          label={"Placements"}
          value={formatCount(description.placements)}
          caption={
            description.restricted
              ? `${formatCount(description.restricted)} only some multiplayer modes load, which a single-player session never plays`
              : "Every one of them plays in a single-player session"
          }
        />

        <ArchiveDescriptionRow
          label={"Effects"}
          value={`${effects.length}`}
          caption={"Each names a definition inside particles.xr rather than a file of its own"}
        />

        <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />
      </EditorPanelSection>

      <EditorPanelSection
        title={`Effects (${effects.length})`}
        caption={"In name order, grouped: a level plants a few effects many times over"}
      >
        {effects.map((effect: ArchiveLevelPsStaticEffect) => (
          <ArchiveLevelPsStaticEffectRow key={effect.name} effect={effect} />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
