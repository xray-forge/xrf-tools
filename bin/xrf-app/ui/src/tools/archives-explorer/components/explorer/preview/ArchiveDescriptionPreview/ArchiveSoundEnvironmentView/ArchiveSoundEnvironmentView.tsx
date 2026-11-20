import { ArchiveSoundEnvironment, ArchiveSoundEnvironmentDescription } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection
        title={`Reverb presets (${environments.length})`}
        caption={"In the order the file holds them, which is how a level addresses one"}
      >
        {environments.map((environment: ArchiveSoundEnvironment) => (
          <EditorPanelProperty
            isInline
            key={environment.name}
            label={environment.name}
            value={describeSpace(environment)}
            caption={describeLevels(environment)}
          />
        ))}
      </EditorPanelSection>
    </ReadingColumn>
  );
}
