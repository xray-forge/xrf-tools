import { useInjection } from "@wirestate/react";
import { TextureDescription } from "@xrf/ipc/types/xrf-app-textures";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty, EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material/chip";
import { ReactElement } from "react";

import {
  describeBumpDeclaration,
  describeBumpInput,
  describeBumpOutcome,
  describeDetail,
  describeVirtualHeight,
  IMaterialStateDescriptor,
} from "@/core/materials/lib";
import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";

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
        <EditorPanelProperty label={"Texture"} value={description.reference} isMonospace />

        <EditorPanelProperty
          label={"Bump"}
          value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
        />

        <EditorPanelProperty
          label={"Declared by"}
          value={declaration ?? "No .thm sits beside this texture in any searched root"}
        />
      </EditorPanelSection>

      {material.bump ? (
        <EditorPanelSection title={"Bump pair"} caption={"What the renderer binds, substitutions included"}>
          <EditorPanelProperty label={"Bump map"} value={describeBumpInput(material.bump.bump)} />
          <EditorPanelProperty label={"Bump#"} value={describeBumpInput(material.bump.companion)} />
          <EditorPanelProperty label={"Height"} value={describeVirtualHeight(material.bump.virtualHeight)} />
        </EditorPanelSection>
      ) : null}

      {material.detail ? (
        <EditorPanelSection title={"Detail"}>
          <EditorPanelProperty label={"Association"} value={describeDetail(material.detail)} />
        </EditorPanelSection>
      ) : null}
    </EditorPanel>
  );
}
