import { ReactElement } from "react";

import {
  ArchiveDescribeScope,
  ArchiveLevelSndStaticDescription,
  ArchiveLevelSndStaticSound,
} from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveLevelSndStaticSoundRow } from "./ArchiveLevelSndStaticSoundRow";

interface IArchiveLevelSndStaticViewProps extends BaseComponentProps {
  description: ArchiveLevelSndStaticDescription;
  scope: ArchiveDescribeScope;
}

/**
 * The sounds a level plants: what plays where nothing has to ask for it.
 */
export function ArchiveLevelSndStaticView({
  "data-testid": dataTestId = "archive-level-snd-static-view",
  id,
  className,
  description,
  scope,
}: IArchiveLevelSndStaticViewProps): ReactElement {
  const { sounds } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Planted sounds"} isFirst>
        <ArchiveDescriptionRow
          label={"Sounds"}
          value={`${sounds.length}`}
          caption={"Each playing on its own, without an object asking for it"}
        />

        <ArchiveDescriptionRow
          label={"On a schedule"}
          value={`${description.scheduled}`}
          caption={"Sounds allowed only inside a window of the day; the rest play at any hour"}
        />
      </EditorPanelSection>

      <EditorPanelSection
        title={`Sounds (${sounds.length})`}
        caption={"In the order the file plants them, which is the order the editor wrote them in"}
      >
        {sounds.map((sound: ArchiveLevelSndStaticSound, index: number) => (
          <ArchiveLevelSndStaticSoundRow key={index} sound={sound} scope={scope} />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
