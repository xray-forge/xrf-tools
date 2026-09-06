import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { EditorPanel, EditorPanelEmpty, EditorPanelRow, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeTextureFile, ITextureFile, selectBoundTextureFiles } from "./TextureFilesPanel.utils";

/**
 * The files behind the selected texture: the base the reference resolves to, and the pair the engine binds.
 */
export function TextureFilesPanel({
  "data-testid": dataTestId = "texture-files-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  const description: Nullable<TextureDescription> = selectionService.selected.value;

  if (!description) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Files"}>
        <EditorPanelEmpty label={"No texture selected. The files behind it show here."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Files"}>
      <EditorPanelSection title={"Bound files"} caption={"What the engine reads for this surface"} isFirst>
        {selectBoundTextureFiles(description).map((file: ITextureFile) => (
          <EditorPanelRow key={file.label} label={file.label} value={describeTextureFile(file)} />
        ))}
      </EditorPanelSection>
    </EditorPanel>
  );
}
