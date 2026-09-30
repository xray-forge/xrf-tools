import { useInjection } from "@wirestate/react";
import { Maybe, Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback, useEffect, useMemo, useState } from "react";

import { LevelSpawnObject, LevelSpawnObjectDetails, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { LevelSpawnDetails } from "@/core/level/components/panels/LevelSpawnPanel/LevelSpawnDetails";
import { LevelSpawnVisibilityToggle } from "@/core/level/components/panels/LevelSpawnPanel/LevelSpawnVisibilityToggle";
import { useLevelHeld } from "@/core/level/components/panels/LevelSpawnPanel/use-level-held";
import { useLevelSpawnDetails } from "@/core/level/components/panels/LevelSpawnPanel/use-level-spawn-details";
import { toLevelFramedGoTo } from "@/core/level/lib/camera/level-camera-frame";
import { ILevelSpawnDelivery } from "@/core/level/lib/render/level-render-protocol";
import { isLevelSpawnReading } from "@/core/level/lib/spawn/level-spawn-report";
import { toLevelSpawnSphere } from "@/core/level/lib/spawn/level-spawn-sphere";
import {
  listLevelSpawnGroupIds,
  TLevelSpawnTreeRow,
  toLevelSpawnObjectId,
  toLevelSpawnTree,
} from "@/core/level/lib/spawn/level-spawn-tree";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelLoadService, LevelRenderService, LevelViewportService, LevelViewService } from "@/core/level/services";
import { EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EmptyListing } from "@/core/ui/layout";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { TreeRowLabel } from "@/core/ui/tree/TreeRowLabel";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { AsyncState } from "@/lib/async-state";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Every object the level's spawn places, `Category -> section -> object`: found by name, section or visual, shown or
 * hidden a category at a time, and gone to.
 */
export function LevelSpawnPanel({
  "data-testid": dataTestId = "level-spawn-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewService: LevelViewService = useInjection(LevelViewService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  const spawn: Nullable<ILevelSpawnDelivery> = useLevelHeld(loadService.spawn);
  const tree: IUseTreeState = useTreeState();
  const { expandAll } = tree;
  const [filter, setFilter] = useState<string>("");
  // Read here rather than in the rows' callback, so a switch from the toolbar redraws the eyes.
  const options: ILevelViewOptions = viewService.options;

  const description: Nullable<LevelSpawnObjectsDescription> = spawn?.objects ?? null;
  const items: Array<ITreeNode<TLevelSpawnTreeRow>> = useMemo(
    () => (description ? toLevelSpawnTree(description, filter) : []),
    [description, filter]
  );
  const byId: ReadonlyMap<string, LevelSpawnObject> = useMemo(
    () =>
      new Map(
        (description?.objects ?? []).map((object: LevelSpawnObject) => [toLevelSpawnObjectId(object.index), object])
      ),
    [description]
  );
  const matched: number = useMemo(
    () => items.reduce((sum: number, it: ITreeNode<TLevelSpawnTreeRow>) => sum + toCount(it.payload), 0),
    [items]
  );

  const selected: Nullable<LevelSpawnObject> = tree.selectedId ? (byId.get(tree.selectedId) ?? null) : null;
  const details: AsyncState<LevelSpawnObjectDetails> = useLevelSpawnDetails(
    loadService.level.value?.selected.sessionId ?? null,
    selected?.index ?? null
  );

  // A filter that matched inside a group opens it, because a closed group answering a query looks like no answer.
  useEffect(() => {
    if (filter.trim()) {
      expandAll(listLevelSpawnGroupIds(items));
    }
  }, [expandAll, filter, items]);

  const goTo = useCallback(
    (object: LevelSpawnObject) =>
      renderService.goTo(
        toLevelFramedGoTo(
          viewportService.camera,
          toLevelSpawnSphere(object, spawn?.models.get(object.visual) ?? null),
          viewService.camera.fieldOfView
        )
      ),
    [renderService, spawn, viewportService, viewService]
  );

  const onSelect = useCallback((item: ITreeNode<TLevelSpawnTreeRow>) => tree.select(item.id), [tree]);

  const onActivate = useCallback(
    (item: ITreeNode<TLevelSpawnTreeRow>) => {
      if (item.payload?.kind === "object") {
        goTo(item.payload.object);
      }
    },
    [goTo]
  );

  const renderLabel = useCallback((item: ITreeNode<TLevelSpawnTreeRow>): ReactNode => {
    const row: Maybe<TLevelSpawnTreeRow> = item.payload;

    return row && row.kind !== "object" ? (
      <TreeRowLabel label={item.label} caption={String(row.count)} captionTitle={`${row.count} spawned`} />
    ) : (
      item.label
    );
  }, []);

  const renderActions = useCallback(
    (item: ITreeNode<TLevelSpawnTreeRow>): ReactNode => {
      const row: Maybe<TLevelSpawnTreeRow> = item.payload;

      if (row?.kind !== "category") {
        return null;
      }

      const { option } = row.entry;

      return (
        <LevelSpawnVisibilityToggle
          entry={row.entry}
          isShown={options[option]}
          onToggle={() => viewService.setOptions({ ...options, [option]: !options[option] })}
        />
      );
    },
    [options, viewService]
  );

  if (!loadService.level.value) {
    return (
      <EditorPanelEmpty
        data-testid={dataTestId}
        id={id}
        className={className}
        label={"No level open. Open one to list what its spawn places."}
      />
    );
  }

  if (!description) {
    return (
      <EditorPanelEmpty
        data-testid={dataTestId}
        id={id}
        className={className}
        label={
          isLevelSpawnReading(loadService.spawnReport)
            ? "Reading the level's spawn."
            : "The level's spawn places nothing the viewer draws."
        }
      />
    );
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex h-full min-h-0 flex-col", className)}>
      <EditorSearchHeader
        title={"Spawn"}
        count={matched}
        query={filter}
        placeholder={"Filter by name, section or visual"}
        ariaLabel={"Filter spawned objects"}
        onClear={() => setFilter("")}
        onQueryChange={setFilter}
      />

      {items.length ? (
        <div className={"min-h-0 grow"}>
          <VirtualizedTree<TLevelSpawnTreeRow>
            ariaLabel={"Spawned objects"}
            className={"h-full"}
            items={items}
            expandedIds={tree.expandedIds}
            selectedId={tree.selectedId}
            renderLabel={renderLabel}
            renderActions={renderActions}
            onToggleExpanded={tree.toggleExpanded}
            onSelect={onSelect}
            onActivate={onActivate}
          />
        </div>
      ) : (
        <EmptyListing label={"No spawned object matches that."} />
      )}

      {selected ? (
        <div className={"max-h-1/2 shrink-0 overflow-y-auto border-t border-divider"}>
          <LevelSpawnDetails
            object={selected}
            visual={description.visuals[selected.visual] ?? ""}
            details={details}
            onGoTo={() => goTo(selected)}
          />
        </div>
      ) : null}
    </div>
  );
}

/** How many objects a row stands for: a group's count, or one. */
function toCount(row: Maybe<TLevelSpawnTreeRow>): number {
  return row && row.kind !== "object" ? row.count : 1;
}
