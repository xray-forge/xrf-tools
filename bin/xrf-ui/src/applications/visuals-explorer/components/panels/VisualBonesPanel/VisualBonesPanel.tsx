import { RichTreeView, TreeViewDefaultItemModelProperties } from "@mui/x-tree-view";
import { useInjection } from "@wirestate/react";
import { ReactElement, SyntheticEvent, useCallback, useMemo } from "react";

import { toBoneTree } from "@/applications/visuals-explorer/components/panels/VisualBonesPanel/VisualBonesPanel.utils";
import { VisualBoneVisibility } from "@/applications/visuals-explorer/components/panels/VisualBonesPanel/VisualBoneVisibility";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { VisualBone } from "@/core/bindings/types/xrf-visual";
import { VisualPanel } from "@/core/visuals/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/core/visuals/components/panels/VisualPanelEmpty";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
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
  const bones: Array<VisualBone> = visualsService.bones;

  const items: Array<TreeViewDefaultItemModelProperties> = useMemo(() => toBoneTree(bones), [bones]);

  const onSelectBone = useCallback(
    (_: Nullable<SyntheticEvent>, name: Nullable<string>) => visualsService.highlightBone(name),
    [visualsService]
  );

  if (!bones.length) {
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
        <RichTreeView
          items={items}
          defaultExpandedItems={items.map((it) => it.id)}
          selectedItems={visualsService.highlightedBone}
          onSelectedItemsChange={onSelectBone}
        />
      </VisualPanelSection>

      <VisualBoneVisibility />
    </VisualPanel>
  );
}
