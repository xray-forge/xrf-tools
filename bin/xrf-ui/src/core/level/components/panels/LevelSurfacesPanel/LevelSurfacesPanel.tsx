import { useInjection } from "@wirestate/react";
import { Maybe, Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback, useEffect, useMemo, useState } from "react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelSurfaceGeometry, NO_LEVEL_SURFACE_GEOMETRY } from "@/core/level/lib/surface/level-surface-geometry";
import {
  ILevelSurfaceSummary,
  listLevelSurfaces,
  listNamedLevelSurfaces,
} from "@/core/level/lib/surface/level-surface-summary";
import {
  listLevelSurfaceGroupIds,
  TLevelSurfaceTreeRow,
  toLevelSurfaceEntryId,
  toLevelSurfaceShaderId,
  toLevelSurfaceTree,
} from "@/core/level/lib/surface/level-surface-tree";
import { listLevelSurfaceDressing } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService, LevelRenderService, LevelViewportService } from "@/core/level/services";
import { describeSurfaceOutcome } from "@/core/materials/lib";
import { EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EmptyListing } from "@/core/ui/layout";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { TreeRowLabel } from "@/core/ui/tree/TreeRowLabel";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { noop } from "@/lib/callbacks/noop";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { LevelSurfaceRow } from "./LevelSurfaceRow";

/**
 * How every surface of the open level is drawn: its shader table's entries by shader, found by shader, texture or id,
 * and the one chosen described under them.
 */
export function LevelSurfacesPanel({
  "data-testid": dataTestId = "level-surfaces-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const tree: IUseTreeState = useTreeState();
  const { expandAll, select } = tree;
  const [filter, setFilter] = useState<string>("");

  const picked: Nullable<TLevelPick> = viewportService.picked;
  const held: ReadonlyArray<number> = loadService.sectorReport.held;
  const surfaces: Maybe<ReadonlyArray<XraySurfaceDescriptor>> = loadService.level.value?.selected.value.surfaces;

  const named: Array<ILevelSurfaceSummary> = useMemo(
    () => listNamedLevelSurfaces(listLevelSurfaces(surfaces ?? [])),
    [surfaces]
  );
  const items: Array<ITreeNode<TLevelSurfaceTreeRow>> = useMemo(
    () => toLevelSurfaceTree(named, filter),
    [named, filter]
  );
  const byId: ReadonlyMap<string, ILevelSurfaceSummary> = useMemo(
    () => new Map(named.map((summary: ILevelSurfaceSummary) => [toLevelSurfaceEntryId(summary.shaderId), summary])),
    [named]
  );
  const matched: number = useMemo(
    () =>
      items.reduce(
        (sum: number, it: ITreeNode<TLevelSurfaceTreeRow>) =>
          sum + (it.payload?.kind === "shader" ? it.payload.count : 0),
        0
      ),
    [items]
  );

  const selected: Nullable<ILevelSurfaceSummary> = tree.selectedId ? (byId.get(tree.selectedId) ?? null) : null;
  const dressing: Array<ILevelSurfaceDressing> = selected
    ? listLevelSurfaceDressing(selected.textures, viewportService.textureReport)
    : [];

  // Measured for the entry chosen, again as the sectors held change: it samples every draw of every one of them.
  const geometry: ILevelSurfaceGeometry = useMemo(
    () =>
      selected
        ? (renderService.measureSurfaceGeometry().get(selected.shaderId) ?? NO_LEVEL_SURFACE_GEOMETRY)
        : NO_LEVEL_SURFACE_GEOMETRY,
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [renderService, held, selected]
  );

  const onSelect = useCallback((item: ITreeNode<TLevelSurfaceTreeRow>) => select(item.id), [select]);

  // Another level's table is numbered afresh, so what was chosen names some other entry now.
  useEffect(() => select(null), [named, select]);

  useEffect(() => {
    const summary: Maybe<ILevelSurfaceSummary> =
      picked?.kind === ELevelPick.SURFACE ? byId.get(toLevelSurfaceEntryId(picked.shaderId)) : undefined;

    if (summary) {
      setFilter("");
      expandAll([toLevelSurfaceShaderId(summary.shader)]);
      select(toLevelSurfaceEntryId(summary.shaderId));
    }
  }, [byId, expandAll, picked, select]);

  // A filter that matched inside a shader opens it, because a closed shader answering a query looks like no answer.
  useEffect(() => {
    if (filter.trim()) {
      expandAll(listLevelSurfaceGroupIds(items));
    }
  }, [expandAll, filter, items]);

  const renderLabel = useCallback((item: ITreeNode<TLevelSurfaceTreeRow>): ReactNode => {
    const row: Maybe<TLevelSurfaceTreeRow> = item.payload;

    if (row?.kind === "shader") {
      return <TreeRowLabel label={item.label} caption={String(row.count)} captionTitle={`${row.count} entries`} />;
    }

    return row ? (
      <TreeRowLabel
        label={item.label}
        caption={describeSurfaceOutcome(row.summary.descriptor).label}
        captionTitle={"Drawn as"}
      />
    ) : (
      item.label
    );
  }, []);

  if (!loadService.level.value) {
    return (
      <EditorPanelEmpty
        data-testid={dataTestId}
        id={id}
        className={className}
        label={"No level open. Open one to see how its surfaces are drawn."}
      />
    );
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex h-full min-h-0 flex-col", className)}>
      <EditorSearchHeader
        title={"Surfaces"}
        count={matched}
        query={filter}
        placeholder={"Filter by shader, texture or id"}
        ariaLabel={"Filter surfaces"}
        onClear={() => setFilter("")}
        onQueryChange={setFilter}
      />

      {items.length ? (
        <div className={"min-h-0 grow"}>
          <VirtualizedTree<TLevelSurfaceTreeRow>
            ariaLabel={"Shader table"}
            className={"h-full"}
            items={items}
            expandedIds={tree.expandedIds}
            selectedId={tree.selectedId}
            renderLabel={renderLabel}
            onToggleExpanded={tree.toggleExpanded}
            onSelect={onSelect}
            onActivate={noop}
          />
        </div>
      ) : (
        <EmptyListing
          label={filter ? "No shader table entry matches that." : "The level's shader table names no shader."}
        />
      )}

      {selected ? (
        <div className={"max-h-1/2 shrink-0 overflow-y-auto border-t border-divider"}>
          <LevelSurfaceRow summary={selected} dressing={dressing} geometry={geometry} isFirst />
        </div>
      ) : null}
    </div>
  );
}
