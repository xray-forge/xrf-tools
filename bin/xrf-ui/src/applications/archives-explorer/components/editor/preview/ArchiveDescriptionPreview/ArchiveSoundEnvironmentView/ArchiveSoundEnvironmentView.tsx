import { ReactElement } from "react";

import { ArchiveSoundEnvironment, ArchiveSoundEnvironmentDescription } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeLevels, describeSpace } from "./ArchiveSoundEnvironmentView.utils";

interface IArchiveSoundEnvironmentViewProps extends BaseComponentProps {
  description: ArchiveSoundEnvironmentDescription;
}

/**
 * The sound environment library: the reverb presets a level puts a listener inside.
 */
export function ArchiveSoundEnvironmentView({
  "data-testid": dataTestId = "archive-sound-environment-view",
  id,
  className,
  description,
}: IArchiveSoundEnvironmentViewProps): ReactElement {
  const { environments } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection
        title={`Reverb presets (${environments.length})`}
        caption={"In the order the file holds them, which is how a level addresses one"}
        isFirst
      >
        {environments.map((environment: ArchiveSoundEnvironment) => (
          <ArchiveDescriptionRow
            key={environment.name}
            label={environment.name}
            value={describeSpace(environment)}
            caption={describeLevels(environment)}
          />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
