import { RichTreeView, TreeViewDefaultItemModelProperties } from "@mui/x-tree-view";
import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { toBoneTree } from "@/applications/visuals-explorer/components/panels/VisualBonesPanel/VisualBonesPanel.utils";
import { VisualPanel } from "@/applications/visuals-explorer/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/applications/visuals-explorer/components/panels/VisualPanelEmpty";
import { VisualPanelSection } from "@/applications/visuals-explorer/components/panels/VisualPanelSection";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { VisualBone } from "@/core/bindings/types/xrf-visual";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** The open visual's skeleton, as the tree its parent names describe. */
export interface IVisualBonesPanelProps extends BaseComponentProps {}

export function VisualBonesPanel({
  "data-testid": dataTestId = "visual-bones-panel",
  id,
  className,
}: IVisualBonesPanelProps = {}): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const bones: Nullable<Array<VisualBone>> = visualsService.visual.value?.selected.description.bones ?? null;

  const items: Array<TreeViewDefaultItemModelProperties> = useMemo(() => toBoneTree(bones ?? []), [bones]);

  if (!bones || bones.length === 0) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Bones"}>
        <VisualPanelEmpty label={"No skeleton. Ogf bone and ik chunks land here."} />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Bones"}>
      <VisualPanelSection
        title={`Skeleton (${bones.length})`}
        caption={"Bone names, parented as ogf stores them"}
        isFirst={true}
      >
        <RichTreeView items={items} defaultExpandedItems={items.map((it) => it.id)} />
      </VisualPanelSection>
    </VisualPanel>
  );
}
