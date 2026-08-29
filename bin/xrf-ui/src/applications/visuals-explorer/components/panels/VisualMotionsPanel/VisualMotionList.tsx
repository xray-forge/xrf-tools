import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { Box, Tooltip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useCallback, useEffect, useMemo } from "react";

import {
  getMotionNodeName,
  groupMotionNames,
  listMotionGroupIds,
} from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/motion-groups";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { VisualPanelEmpty } from "@/core/visuals/components/panels";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IVisualMotionListProps extends BaseComponentProps {
  /** What the panel's filter field holds, already the user's whole query. */
  filter: string;
}

/**
 * Every motion the open visual can play, grouped by the token its name starts with.
 */
export function VisualMotionList({
  "data-testid": dataTestId = "visual-motion-list",
  id,
  className,
  filter,
}: IVisualMotionListProps): ReactElement {
  const service: VisualMotionService = useInjection(VisualMotionService);
  const tree: IUseTreeState = useTreeState();
  const { expandAll } = tree;

  // The loadable's own value is what the memo depends on: a default of `[]` is a fresh array every render, which as a
  // dependency would regroup on each one.
  const listed: Nullable<Array<string>> = service.motions.value;
  const posed: Nullable<string> = service.posed.value?.bake.name ?? null;

  const matched: Array<string> = useMemo(() => {
    const needle: string = filter.trim().toLowerCase();

    return needle ? (listed ?? []).filter((name: string) => name.toLowerCase().includes(needle)) : (listed ?? []);
  }, [filter, listed]);

  const nodes: Array<ITreeNode<string>> = useMemo(() => groupMotionNames(matched), [matched]);

  /** Marks the motion the viewport is posing, which selection cannot say: selection is where the user is. */
  const renderLabel = useCallback(
    (item: ITreeNode<string>): ReactNode =>
      item.payload === posed ? (
        <Box sx={{ alignItems: "center", display: "flex", gap: 0.5, minWidth: 0 }}>
          <Box component={"span"} sx={{ minWidth: 0, overflow: "hidden", textOverflow: "ellipsis" }}>
            {item.label}
          </Box>

          <Tooltip title={"Posed in the viewport"}>
            <PlayArrowIcon fontSize={"inherit"} sx={{ color: "primary.main", flexShrink: 0 }} />
          </Tooltip>
        </Box>
      ) : (
        item.label
      ),
    [posed]
  );

  const onSelect = useCallback((item: ITreeNode<string>) => tree.select(item.id), [tree]);

  /** Posing reads and bakes a motion, which is work, so it waits for the gesture that means work. */
  const onActivate = useCallback(
    (item: ITreeNode<string>) => {
      const name: Nullable<string> = getMotionNodeName(item.id);

      if (name) {
        void service.open(name);
      }
    },
    [service]
  );

  // A filter that matched inside a family opens it, because a closed family answering a query looks like no answer.
  // Additive, so a family opened by hand stays open once the query is cleared.
  useEffect(() => {
    if (filter.trim()) {
      expandAll(listMotionGroupIds(nodes));
    }
  }, [expandAll, filter, nodes]);

  if (service.motions.isLoading) {
    return <VisualPanelEmpty label={"Listing motions. Every animation file the visual references is read once."} />;
  }

  if (!listed?.length) {
    return (
      <VisualPanelEmpty
        label={service.motions.error?.message ?? "This visual references animation files that name no motions."}
      />
    );
  }

  if (!nodes.length) {
    return <VisualPanelEmpty label={`No motion of the ${listed.length} this visual plays matches that.`} />;
  }

  return (
    <VirtualizedTree<string>
      data-testid={dataTestId}
      id={id}
      className={className}
      ariaLabel={"Motions"}
      items={nodes}
      expandedIds={tree.expandedIds}
      selectedId={tree.selectedId}
      renderLabel={renderLabel}
      sx={{ minHeight: 160, padding: 0 }}
      onSelect={onSelect}
      onActivate={onActivate}
      onToggleExpanded={tree.toggleExpanded}
    />
  );
}
