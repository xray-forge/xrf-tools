import { ReactElement } from "react";

import { VisualBounds } from "@/core/bindings/types/xrf-visual";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
import { formatCoordinate, formatVector } from "@/core/visuals/lib/visual-format";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IVisualBoundsSectionProps extends BaseComponentProps {
  title: string;
  /** Which of the two extents this is, since the rows themselves are identical. */
  caption: string;
  bounds: Nullable<VisualBounds>;
}

/** One extent, as a box and the sphere around it. */
export function VisualBoundsSection({
  "data-testid": dataTestId = "visual-bounds-section",
  id,
  className,
  title,
  caption,
  bounds,
}: IVisualBoundsSectionProps): ReactElement {
  return (
    <VisualPanelSection data-testid={dataTestId} id={id} className={className} title={title} caption={caption}>
      <VisualPanelRow label={"Min"} value={formatVector(bounds?.boundingBox.min ?? null)} />
      <VisualPanelRow label={"Max"} value={formatVector(bounds?.boundingBox.max ?? null)} />
      <VisualPanelRow label={"Centre"} value={formatVector(bounds?.boundingSphere.center ?? null)} />
      <VisualPanelRow label={"Radius"} value={formatCoordinate(bounds?.boundingSphere.radius ?? null)} />
    </VisualPanelSection>
  );
}
