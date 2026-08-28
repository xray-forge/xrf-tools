import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { VisualBone } from "@/core/bindings/types/xrf-visual";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { IVisualInspection, VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { toBoneTree } from "@/core/visuals/components/panels/VisualBonesPanel/VisualBonesPanel.utils";
import { VisualBoneVisibility } from "@/core/visuals/components/panels/VisualBonesPanel/VisualBoneVisibility";
import { VisualPanel } from "@/core/visuals/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/core/visuals/components/panels/VisualPanelEmpty";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** The open visual's skeleton, as the tree its parent names describe. */
export interface IVisualBonesPanelProps extends BaseComponentProps {}

/** Height the skeleton keeps for itself before the panel starts scrolling instead. */
const TREE_MIN_HEIGHT: number = 160;

export function VisualBonesPanel({
  "data-testid": dataTestId = "visual-bones-panel",
  id,
  className,
}: IVisualBonesPanelProps = {}): ReactElement {
  const { bones, boneControls }: IVisualInspection = useInjection(VISUAL_INSPECTION);

  const tree: IUseTreeState = useTreeState();

  const items: Array<ITreeNode<VisualBone>> = useMemo(() => toBoneTree(bones), [bones]);

  const { expandAll } = tree;

  // Highlighting is how the selected bone is drawn - in the viewport rather than only in the list - so it follows
  // selection rather than waiting for an activation. A surface offering no controls simply does not select.
  const onSelectBone = useCallback(
    (node: ITreeNode<VisualBone>) => boneControls?.highlightBone(node.id),
    [boneControls]
  );

  // A bone has nothing to open: activating one only folds it, which the tree has already done by this point.
  const onActivateBone = useCallback(() => undefined, []);

  useEffect(() => expandAll(items.map((it: ITreeNode<VisualBone>) => it.id)), [expandAll, items]);

  if (!bones.length) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Bones"}>
        <VisualPanelEmpty label={"No skeleton. Ogf bone and ik chunks land here."} />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Bones"}
      // At least the panel's height, so the skeleton can take what the switches below it leave; more than that
      // when they need it, and the panel scrolls as it did before.
      sx={{ minHeight: "100%" }}
    >
      {/* The panel's own content slot is a block, so the column the skeleton and the switches divide starts here. */}
      <Box sx={{ display: "flex", flexDirection: "column", height: "100%" }}>
        <VisualPanelSection
          title={`Skeleton (${bones.length})`}
          caption={"Bone names, parented as ogf stores them"}
          isFirst={true}
          isFilling={true}
        >
          <VirtualizedTree<VisualBone>
            ariaLabel={"Skeleton bones"}
            items={items}
            expandedIds={tree.expandedIds}
            selectedId={boneControls?.highlightedBone ?? null}
            sx={{ minHeight: TREE_MIN_HEIGHT, padding: 0 }}
            onSelect={onSelectBone}
            onActivate={onActivateBone}
            onToggleExpanded={tree.toggleExpanded}
          />
        </VisualPanelSection>

        <VisualBoneVisibility />
      </Box>
    </VisualPanel>
  );
}
