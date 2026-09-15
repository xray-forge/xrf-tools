import { ReactElement } from "react";

import { ArchiveThmParameters } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { formatColorWord, NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveThmFlagList } from "./ArchiveThmFlagList";

interface IArchiveThmRecipeSectionProps extends BaseComponentProps {
  parameters: Nullable<ArchiveThmParameters>;
  fadeDelay: Nullable<number>;
}

/**
 * What the converter was told to do, and has already carried out.
 */
export function ArchiveThmRecipeSection({
  "data-testid": dataTestId = "archive-thm-recipe-section",
  id,
  className,
  parameters,
  fadeDelay,
}: IArchiveThmRecipeSectionProps): ReactElement {
  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Build recipe"}
      caption={"What the converter was told to do, which it has already carried out"}
    >
      {parameters ? (
        <>
          <ArchiveDescriptionRow label={"Format"} value={parameters.formatLabel} />

          <ArchiveDescriptionRow label={"Mip filter"} value={parameters.mipFilterLabel} />

          <ArchiveDescriptionRow
            label={"Converted size"}
            value={`${parameters.width} × ${parameters.height}`}
            isMonospace
          />

          <ArchiveDescriptionRow label={"Border colour"} value={formatColorWord(parameters.borderColor)} isMonospace />

          <ArchiveDescriptionRow label={"Fade colour"} value={formatColorWord(parameters.fadeColor)} isMonospace />

          <ArchiveDescriptionRow label={"Fade amount"} value={parameters.fadeAmount} />

          <ArchiveDescriptionRow
            label={"Fade delay"}
            value={fadeDelay === null ? NOT_DECLARED : fadeDelay}
            caption={"Mip level the converter starts fading from"}
          />

          <ArchiveDescriptionRow label={"Flags"} value={<ArchiveThmFlagList flags={parameters.flags} />} />

          {parameters.unnamedFlags ? (
            <ArchiveDescriptionRow
              label={"Unnamed bits"}
              value={formatColorWord(parameters.unnamedFlags)}
              caption={"Bits the flag word carries that the SDK has no name for"}
              isMonospace
            />
          ) : null}
        </>
      ) : (
        <ArchiveDescriptionRow label={"Parameters"} value={NOT_DECLARED} />
      )}
    </EditorPanelSection>
  );
}
