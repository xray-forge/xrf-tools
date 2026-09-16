import { ReactElement } from "react";

import { describeAssetContainer, isLooseContainer } from "@/core/assets/lib";
import { ArchiveShadowedCopy, ArchiveWorldEntry } from "@/core/ipc/types/xrf-app";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

interface IArchiveOriginDetailProps extends BaseComponentProps {
  entry: ArchiveWorldEntry;
}

/**
 * Where the selected file's bytes actually come from, and which copies that decision hides.
 */
export function ArchiveOriginDetail({
  "data-testid": dataTestId = "archive-origin-detail",
  id,
  className,
  entry,
}: IArchiveOriginDetailProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Origin"}>
      <EditorPanelProperty label={"Read from"} value={isLooseContainer(entry.container) ? "Loose file" : "Archive"} />

      <EditorPanelProperty label={"Path"} value={describeAssetContainer(entry.container)} isMonospace />

      <EditorPanelProperty
        label={"Overrides"}
        value={
          entry.shadowed.length
            ? `${entry.shadowed.length} other cop${entry.shadowed.length === 1 ? "y" : "ies"} of this engine path`
            : "Nothing - this path is held once"
        }
      />

      {entry.shadowed.map((copy: ArchiveShadowedCopy) => (
        <EditorPanelProperty
          key={describeAssetContainer(copy.container)}
          label={isLooseContainer(copy.container) ? "Hidden loose file" : "Hidden archive entry"}
          value={`${describeAssetContainer(copy.container)} (${formatBytes(copy.sizeReal)})`}
          isMonospace
        />
      ))}
    </EditorPanelSection>
  );
}
