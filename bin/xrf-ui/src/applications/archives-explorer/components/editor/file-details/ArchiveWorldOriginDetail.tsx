import { ReactElement } from "react";

import { describeAssetContainer, isLooseContainer } from "@/core/assets/lib";
import { ArchiveWorldEntry } from "@/core/ipc/types/xrf-app";
import { XrayAssetContainer } from "@/core/ipc/types/xrf-vfs";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveWorldOriginDetailProps extends BaseComponentProps {
  entry: ArchiveWorldEntry;
}

/**
 * Where the selected file's bytes actually come from, and which copies that decision hides.
 */
export function ArchiveWorldOriginDetail({
  "data-testid": dataTestId = "archive-world-origin-detail",
  id,
  className,
  entry,
}: IArchiveWorldOriginDetailProps): ReactElement {
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

      {entry.shadowed.map((container: XrayAssetContainer) => (
        <EditorPanelProperty
          key={describeAssetContainer(container)}
          label={isLooseContainer(container) ? "Hidden loose file" : "Hidden archive entry"}
          value={describeAssetContainer(container)}
          isMonospace
        />
      ))}
    </EditorPanelSection>
  );
}
