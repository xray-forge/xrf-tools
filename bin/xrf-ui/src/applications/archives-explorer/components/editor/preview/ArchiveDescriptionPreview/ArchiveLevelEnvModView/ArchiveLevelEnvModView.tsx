import { ReactElement } from "react";

import { ArchiveLevelEnvModDescription, ArchiveLevelEnvModifier } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveLevelEnvModifierSection } from "./ArchiveLevelEnvModifierSection";

interface IArchiveLevelEnvModViewProps extends BaseComponentProps {
  description: ArchiveLevelEnvModDescription;
}

/**
 * A level's local weather overrides: where the sky stops being the sky the weather cycle asked for.
 */
export function ArchiveLevelEnvModView({
  "data-testid": dataTestId = "archive-level-env-mod-view",
  id,
  className,
  description,
}: IArchiveLevelEnvModViewProps): ReactElement {
  const { modifiers } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Weather overrides"} isFirst>
        <ArchiveDescriptionRow
          label={"Overrides"}
          value={`${modifiers.length}`}
          caption={
            modifiers.length
              ? "Each a sphere the weather cycle is mixed with inside"
              : "The weather cycle stands everywhere on this level"
          }
        />

        <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />
      </EditorPanelSection>

      {modifiers.map((modifier: ArchiveLevelEnvModifier, index: number) => (
        <ArchiveLevelEnvModifierSection key={index} modifier={modifier} index={index} />
      ))}
    </ArchiveDescriptionLayout>
  );
}
