import { formatCount } from "@xrf/format";
import { ArchiveLevelPsStaticDescription, ArchiveLevelPsStaticEffect } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Planted effects"}>
        <EditorPanelProperty
          isInline
          label={"Placements"}
          value={formatCount(description.placements)}
          caption={
            description.restricted
              ? `${formatCount(description.restricted)} only some multiplayer modes load, which a single-player session never plays`
              : "Every one of them plays in a single-player session"
          }
        />

        <EditorPanelProperty
          isInline
          label={"Effects"}
          value={`${effects.length}`}
          caption={"Each names a definition inside particles.xr rather than a file of its own"}
        />

        <EditorPanelProperty isInline label={"Version"} value={`${description.version}`} />
      </EditorPanelSection>

      <EditorPanelSection
        title={`Effects (${effects.length})`}
        caption={"In name order, grouped: a level plants a few effects many times over"}
      >
        {effects.map((effect: ArchiveLevelPsStaticEffect) => (
          <ArchiveLevelPsStaticEffectRow key={effect.name} effect={effect} />
        ))}
      </EditorPanelSection>
    </ReadingColumn>
  );
}
