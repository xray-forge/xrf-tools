import { Chip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TextureDescription } from "@/core/bindings/types/xrf-app";
import {
  describeBumpDeclaration,
  describeBumpInput,
  describeBumpOutcome,
  describeDetail,
  describeVirtualHeight,
  IMaterialStateDescriptor,
} from "@/core/materials/lib";
import { EditorPanel, EditorPanelEmpty, EditorPanelRow, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * What the game builds for the selected texture, read from its `.thm` the way the engine reads it.
 */
export function TextureMaterialPanel({
  "data-testid": dataTestId = "texture-material-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  const description: Nullable<TextureDescription> = selectionService.selected.value;

  if (!description) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Material"}>
        <EditorPanelEmpty label={"No texture selected. What its descriptor declares shows here."} />
      </EditorPanel>
    );
  }

  const { material } = description;

  if (!material) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Material"}>
        <EditorPanelEmpty
          label={
            "This file sits outside a game tree, so there is nothing to resolve a bump pair or a detail against. " +
            "What its descriptor declares is in the Descriptor panel."
          }
        />
      </EditorPanel>
    );
  }

  const outcome: IMaterialStateDescriptor = describeBumpOutcome(material.outcome);
  const declaration: Nullable<string> = describeBumpDeclaration(material.declaration, material.descriptor);

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Material"}>
      <EditorPanelSection title={"Declaration"} isFirst>
        <EditorPanelRow label={"Texture"} value={description.reference} />

        <EditorPanelRow
          label={"Bump"}
          value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
        />

        <EditorPanelRow
          label={"Declared by"}
          value={declaration ?? "No .thm sits beside this texture in any searched root"}
        />
      </EditorPanelSection>

      {material.bump ? (
        <EditorPanelSection title={"Bump pair"} caption={"What the renderer binds, substitutions included"}>
          <EditorPanelRow label={"Bump map"} value={describeBumpInput(material.bump.bump)} />
          <EditorPanelRow label={"Bump#"} value={describeBumpInput(material.bump.companion)} />
          <EditorPanelRow label={"Height"} value={describeVirtualHeight(material.bump.virtualHeight)} />
        </EditorPanelSection>
      ) : null}

      {material.detail ? (
        <EditorPanelSection title={"Detail"}>
          <EditorPanelRow label={"Association"} value={describeDetail(material.detail)} />
        </EditorPanelSection>
      ) : null}
    </EditorPanel>
  );
}
