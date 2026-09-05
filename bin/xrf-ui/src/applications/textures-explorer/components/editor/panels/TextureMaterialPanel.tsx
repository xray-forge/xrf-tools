import { Chip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import {
  describeBumpDeclaration,
  describeBumpInput,
  describeBumpOutcome,
  describeDetail,
  describeVirtualHeight,
  IMaterialStateDescriptor,
} from "@/core/materials/lib";
import { VisualPanel } from "@/core/visuals/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/core/visuals/components/panels/VisualPanelEmpty";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
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
  const texturesService: TexturesService = useInjection(TexturesService);

  const description: Nullable<TextureDescription> = texturesService.selected.value;

  if (!description) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Material"}>
        <VisualPanelEmpty label={"No texture selected. What its descriptor declares shows here."} />
      </VisualPanel>
    );
  }

  const { material } = description;
  const outcome: IMaterialStateDescriptor = describeBumpOutcome(material.outcome);
  const declaration: Nullable<string> = describeBumpDeclaration(material.declaration, material.descriptor);

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Material"}>
      <VisualPanelSection title={"Declaration"} isFirst>
        <VisualPanelRow label={"Texture"} value={description.reference} />

        <VisualPanelRow
          label={"Bump"}
          value={<Chip size={"small"} color={outcome.color} variant={"outlined"} label={outcome.label} />}
        />

        <VisualPanelRow
          label={"Declared by"}
          value={declaration ?? "No .thm sits beside this texture in any searched root"}
        />
      </VisualPanelSection>

      {material.bump ? (
        <VisualPanelSection title={"Bump pair"} caption={"What the renderer binds, substitutions included"}>
          <VisualPanelRow label={"Bump map"} value={describeBumpInput(material.bump.bump)} />
          <VisualPanelRow label={"Bump#"} value={describeBumpInput(material.bump.companion)} />
          <VisualPanelRow label={"Height"} value={describeVirtualHeight(material.bump.virtualHeight)} />
        </VisualPanelSection>
      ) : null}

      {material.detail ? (
        <VisualPanelSection title={"Detail"}>
          <VisualPanelRow label={"Association"} value={describeDetail(material.detail)} />
        </VisualPanelSection>
      ) : null}
    </VisualPanel>
  );
}
