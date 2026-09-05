import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { VisualPanel } from "@/core/visuals/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/core/visuals/components/panels/VisualPanelEmpty";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
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
  const texturesService: TexturesService = useInjection(TexturesService);

  const description: Nullable<TextureDescription> = texturesService.selected.value;

  if (!description) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Files"}>
        <VisualPanelEmpty label={"No texture selected. The files behind it show here."} />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Files"}>
      <VisualPanelSection title={"Bound files"} caption={"What the engine reads for this surface"} isFirst>
        {selectBoundTextureFiles(description).map((file: ITextureFile) => (
          <VisualPanelRow key={file.label} label={file.label} value={describeTextureFile(file)} />
        ))}
      </VisualPanelSection>
    </VisualPanel>
  );
}
