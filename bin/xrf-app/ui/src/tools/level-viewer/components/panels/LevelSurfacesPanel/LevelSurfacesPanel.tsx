import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { Maybe, Nullable } from "@xrf/types";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EmptyListing } from "@xrf/ui/layout/empty-listing";
import { cn } from "@xrf/ui/theme";
import { ITreeNode, IUseTreeState, TreeRowLabel, useTreeState, VirtualizedTree } from "@xrf/ui/tree";
import { ReactElement, ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { EMPTY_LEVEL_RESIDENT, ILevelResident } from "@/core/level/lib/render/level-resident";
import { ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelSurfaceGeometry, NO_LEVEL_SURFACE_GEOMETRY } from "@/core/level/lib/surface/level-surface-geometry";
import {
  findLevelSurface,
  ILevelSurfaceSummary,
  listLevelSurfaces,
  listNamedLevelSurfaces,
} from "@/core/level/lib/surface/level-surface-summary";
import { listLevelSurfaceDressing } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { describeSurfaceOutcome } from "@/core/materials/lib";
import { EditorSearchHeader } from "@/core/shell/chrome/EditorSearchHeader";
import { noop } from "@/lib/callbacks/noop";
import { describeLevelResidentWait } from "@/tools/level-viewer/lib/load/level-resident-wait";
import {
  listLevelSurfaceGroupIds,
  TLevelSurfaceTreeRow,
  toLevelSurfaceEntryId,
  toLevelSurfaceShaderId,
  toLevelSurfaceTree,
} from "@/tools/level-viewer/lib/surface/level-surface-tree";

import { LevelSurfaceDetails } from "./LevelSurfaceDetails";

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
  const isPickRelocated: boolean = viewportService.isPickRelocated;
  const surfaces: Nullable<ReadonlyArray<XraySurfaceDescriptor>> = loadService.surfaces.value;

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

  const resident: ILevelResident = renderService.resident.value ?? EMPTY_LEVEL_RESIDENT;
  const selected: Nullable<ILevelSurfaceSummary> = tree.selectedId ? (byId.get(tree.selectedId) ?? null) : null;
  const dressing: Array<ILevelSurfaceDressing> = selected
    ? listLevelSurfaceDressing(selected.textures, resident.textures)
    : [];

  const geometry: ILevelSurfaceGeometry = selected
    ? (resident.surfaces.get(selected.shaderId) ?? NO_LEVEL_SURFACE_GEOMETRY)
    : NO_LEVEL_SURFACE_GEOMETRY;

  // The entry chosen as the table it was chosen in read it, which a table read anew finds again; null for a shader's
  // row, which its name keeps.
  const chosen = useRef<Nullable<XraySurfaceDescriptor>>(null);
  // The pick last heard, so the tree answers each once.
  const heard = useRef<Nullable<TLevelPick>>(null);
  // The table the entry was chosen in.
  const read = useRef<Nullable<ReadonlyArray<XraySurfaceDescriptor>>>(surfaces);
  const sessionId: Nullable<string> = loadService.level.value?.selected.sessionId ?? null;

  const choose = useCallback(
    (id: Nullable<string>, entry: Nullable<XraySurfaceDescriptor>) => {
      chosen.current = entry;
      select(id);
    },
    [select]
  );

  const onSelect = useCallback(
    (item: ITreeNode<TLevelSurfaceTreeRow>) =>
      choose(item.id, item.payload?.kind === "entry" ? item.payload.summary.descriptor : null),
    [choose]
  );

  const renderLabel = useCallback((item: ITreeNode<TLevelSurfaceTreeRow>): ReactNode => {
    const row: Maybe<TLevelSurfaceTreeRow> = item.payload;

    if (row?.kind === "shader") {
      return (
        <TreeRowLabel
          label={item.label}
          caption={formatCount(row.count)}
          captionTitle={`${formatCount(row.count)} entries`}
        />
      );
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

  // What was chosen belongs to the level it was chosen in.
  useEffect(() => choose(null, null), [sessionId, choose]);

  // A table read anew may number the entry chosen afresh: it is chosen where it stands now, or nothing where it is gone.
  useEffect(() => {
    const before: Nullable<ReadonlyArray<XraySurfaceDescriptor>> = read.current;

    read.current = surfaces;

    if (!chosen.current || !before || surfaces === before) {
      return;
    }

    const shaderId: Nullable<number> = surfaces ? findLevelSurface(chosen.current, surfaces) : null;

    if (surfaces && shaderId !== null) {
      choose(toLevelSurfaceEntryId(shaderId), surfaces[shaderId] ?? null);
    } else {
      choose(null, null);
    }
  }, [choose, surfaces]);

  // The tree shows what is picked: the entry drawing a surface picked chosen, its shader opened and any filter hiding it
  // cleared; anything else picked, or nothing, chooses nothing here. A pick found again in a table read anew leaves the
  // tree to follow what it chose itself.
  useEffect(() => {
    // A surface picked before the table is read is chosen once it is.
    if (picked === heard.current || isPickRelocated || (picked?.kind === ELevelPick.SURFACE && !surfaces)) {
      return;
    }

    heard.current = picked;

    const summary: Maybe<ILevelSurfaceSummary> =
      picked?.kind === ELevelPick.SURFACE ? byId.get(toLevelSurfaceEntryId(picked.shaderId)) : undefined;

    if (summary) {
      setFilter("");
      expandAll([toLevelSurfaceShaderId(summary.shader)]);
      choose(toLevelSurfaceEntryId(summary.shaderId), summary.descriptor);
    } else {
      choose(null, null);
    }
  }, [byId, choose, expandAll, isPickRelocated, picked, surfaces]);

  // A filter that matched inside a shader opens it, because a closed shader answering a query looks like no answer.
  useEffect(() => {
    if (filter.trim()) {
      expandAll(listLevelSurfaceGroupIds(items));
    }
  }, [expandAll, filter, items]);

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
          <LevelSurfaceDetails
            summary={selected}
            dressing={dressing}
            geometry={geometry}
            unmeasured={describeLevelResidentWait(renderService.resident.state, "how much it draws")}
          />
        </div>
      ) : null}
    </div>
  );
}
