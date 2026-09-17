import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { VisualBone } from "@/core/ipc/types/xrf-visual";
import { EditorPanel, EditorPanelEmpty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { IVisualInspection, VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { toBoneTree } from "@/core/visuals/components/panels/VisualBonesPanel/VisualBonesPanel.utils";
import { VisualBoneVisibility } from "@/core/visuals/components/panels/VisualBonesPanel/VisualBoneVisibility";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export function VisualBonesPanel({
  "data-testid": dataTestId = "visual-bones-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
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
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Bones"}>
        <EditorPanelEmpty label={"No skeleton. Ogf bone and ik chunks land here."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={cn("min-h-full", className)} title={"Bones"}>
      <div className={"flex h-full flex-col"}>
        <EditorPanelSection
          title={`Skeleton (${bones.length})`}
          caption={"Bone names, parented as ogf stores them"}
          isFirst={true}
          isFilling={true}
        >
          <VirtualizedTree
            ariaLabel={"Skeleton bones"}
            className={"min-h-40"}
            items={items}
            expandedIds={tree.expandedIds}
            selectedId={boneControls?.highlightedBone ?? null}
            sx={{ padding: 0 }}
            onSelect={onSelectBone}
            onActivate={onActivateBone}
            onToggleExpanded={tree.toggleExpanded}
          />
        </EditorPanelSection>

        <VisualBoneVisibility />
      </div>
    </EditorPanel>
  );
}
