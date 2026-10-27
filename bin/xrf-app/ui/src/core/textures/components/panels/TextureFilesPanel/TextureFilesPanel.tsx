import { useInjection } from "@wirestate/react";
import { TextureDescription } from "@xrf/ipc/types/xrf-app-textures";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { AssetTextureDetails } from "@/core/assets/components/AssetTextureDetails";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";

import { ITextureFile, selectBoundTextureFiles } from "./TextureFilesPanel.utils";

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
      {selectBoundTextureFiles(description).map((file: ITextureFile, index: number) => (
        <EditorPanelSection key={file.label} title={file.label} isFirst={index === 0}>
          {file.asset ? (
            <AssetTextureDetails asset={file.asset} descriptor={file.descriptor} />
          ) : (
            <EditorPanelProperty label={"Status"} value={"Not bound"} />
          )}
        </EditorPanelSection>
      ))}
    </EditorPanel>
  );
}
