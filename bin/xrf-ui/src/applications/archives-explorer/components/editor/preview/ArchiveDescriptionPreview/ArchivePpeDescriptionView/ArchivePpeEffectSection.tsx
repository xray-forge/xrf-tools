import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchivePpeDescription } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatSeconds } from "@/lib/format/duration";

import { ArchiveDescriptionReference } from "../ArchiveDescriptionReference";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeKeyedParameters } from "./ArchivePpeDescriptionView.utils";

interface IArchivePpeEffectSectionProps extends BaseComponentProps {
  description: ArchivePpeDescription;
  scope: ArchiveDescribeScope;
}

/**
 * What the effect is, taken over the whole of it.
 */
export function ArchivePpeEffectSection({
  "data-testid": dataTestId = "archive-ppe-effect-section",
  id,
  className,
  description,
  scope,
}: IArchivePpeEffectSectionProps): ReactElement {
  const { colorMap } = description;

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Effect"} isFirst>
      <ArchiveDescriptionRow
        label={"Runs for"}
        value={formatSeconds(description.lengthSeconds)}
        caption={"Its longest parameter decides it, not where its last key sits"}
      />

      <ArchiveDescriptionRow
        label={"Keys"}
        value={`${description.keys}`}
        caption={describeKeyedParameters(description)}
      />

      <ArchiveDescriptionRow
        label={"Version"}
        value={`${description.version}`}
        caption={description.version < 2 ? "Below version 2, which is where colour grading was added" : null}
      />

      {colorMap?.texture ? (
        <ArchiveDescriptionReference label={"Colour grading"} reference={colorMap.texture} scope={scope} />
      ) : null}

      {colorMap && !colorMap.isUsed ? (
        <ArchiveDescriptionRow
          label={"Colour grading"}
          value={"Not declared"}
          caption={"The effect carries the parameter but names no gradient, so it grades nothing"}
        />
      ) : null}
    </EditorPanelSection>
  );
}
