import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { LevelSpawnObject, LevelSpawnObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { XrayRevision } from "@xrf/ipc/types/xrf-vfs";
import { Maybe, Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { LevelSpawnDetails } from "@/applications/level-viewer/components/panels/LevelSpawnPanel/LevelSpawnDetails";
import { LevelSpawnVisibilityToggle } from "@/applications/level-viewer/components/panels/LevelSpawnPanel/LevelSpawnVisibilityToggle";
import { describeLevelSpawnRelease } from "@/applications/level-viewer/lib/spawn/level-spawn-release";
import {
  listLevelSpawnGroupIds,
  TLevelSpawnTreeRow,
  toLevelSpawnCategoryId,
  toLevelSpawnObjectId,
  toLevelSpawnSectionId,
  toLevelSpawnTree,
} from "@/applications/level-viewer/lib/spawn/level-spawn-tree";
import { LevelViewerSpawnService } from "@/applications/level-viewer/services/level-viewer-spawn";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { ILevelSpawnReport, isLevelSpawnReading } from "@/core/level/lib/spawn/level-spawn-report";
import { toLevelSpawnPosition } from "@/core/level/lib/spawn/level-spawn-sphere";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelLoadService } from "@/core/level/services/level-load/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport/level-viewport.service";
import { EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { cn } from "@/core/theme/class-name";
import { EmptyListing } from "@/core/ui/layout";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { TreeRowLabel } from "@/core/ui/tree/TreeRowLabel";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * What the panel says while it lists nothing: why the spawn could not be read, that it is being read, or that it
 * places nothing the viewer draws.
 */
function toEmptyLabel(report: ILevelSpawnReport): string {
  if (report.failure) {
    return `The level's spawn could not be read: ${report.failure}`;
  }

  return !report.isListed || isLevelSpawnReading(report)
    ? "Reading the level's spawn."
    : "The level's spawn places nothing the viewer draws.";
}

/** How many objects a row stands for: a group's count, or one. */
function toCount(row: Maybe<TLevelSpawnTreeRow>): number {
  return row && row.kind !== "object" ? row.count : 1;
}

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
  const spawnService: LevelViewerSpawnService = useInjection(LevelViewerSpawnService);

  const tree: IUseTreeState = useTreeState();
  const { expandAll, select } = tree;
  const [filter, setFilter] = useState<string>("");
  // Read here rather than in the rows' callback, so a switch from the toolbar redraws the eyes.
  const options: ILevelViewOptions = viewService.options;
  const picked: Nullable<TLevelPick> = viewportService.picked;
  const isPickRelocated: boolean = viewportService.isPickRelocated;

  // A spawn placing nothing reads as one not listed yet would: the empty state says which.
  const description: Nullable<LevelSpawnObjectsDescription> = loadService.spawn?.objects.length
    ? loadService.spawn
    : null;
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
  const selectedIndex: Nullable<number> = selected?.index ?? null;
  const sessionId: Nullable<string> = loadService.level.value?.selected.sessionId ?? null;
  const spawnRevision: Nullable<XrayRevision> = loadService.spawnRevision;

  // The pick last heard, so the tree answers each once.
  const heard = useRef<Nullable<TLevelPick>>(null);
  // The pick the object chosen shows: made of the row chosen here, or picked elsewhere and followed; null for none.
  const shown = useRef<Nullable<TLevelPick>>(null);

  // An object chosen here is picked as a click on it is, so the viewport marks it and the readout names it.
  const onSelect = useCallback(
    (item: ITreeNode<TLevelSpawnTreeRow>) => {
      select(item.id);
      shown.current = null;

      if (item.payload?.kind === "object" && description) {
        const { object } = item.payload;

        shown.current = {
          kind: ELevelPick.SPAWN,
          object,
          point: toLevelSpawnPosition(object.transform),
          visual: description.visuals[object.visual] ?? "",
        };
        viewportService.notePicked(shown.current);
      }
    },
    [description, select, viewportService]
  );

  const onActivate = useCallback(
    (item: ITreeNode<TLevelSpawnTreeRow>) => {
      if (item.payload?.kind === "object") {
        void spawnService.goTo(item.payload.object);
      }
    },
    [spawnService]
  );

  const onGoToSelected = useCallback(() => {
    if (selected) {
      void spawnService.goTo(selected);
    }
  }, [selected, spawnService]);

  const renderLabel = useCallback((item: ITreeNode<TLevelSpawnTreeRow>): ReactNode => {
    const row: Maybe<TLevelSpawnTreeRow> = item.payload;

    if (!row) {
      return item.label;
    }

    if (row.kind === "object") {
      return row.object.release ? (
        <span className={"opacity-55"} title={describeLevelSpawnRelease(row.object.release)}>
          {item.label}
        </span>
      ) : (
        item.label
      );
    }

    return (
      <TreeRowLabel
        label={item.label}
        caption={
          row.released ? `${formatCount(row.count)} · ${formatCount(row.released)} released` : formatCount(row.count)
        }
        captionTitle={
          row.released
            ? `${formatCount(row.count)} spawned, ${formatCount(row.released)} of them released by a new game`
            : `${formatCount(row.count)} spawned`
        }
      />
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
          onToggle={() => viewService.toggleOption(option)}
        />
      );
    },
    [options, viewService]
  );

  // What was chosen belongs to the level it was chosen in.
  useEffect(() => {
    shown.current = null;
    select(null);
  }, [sessionId, select]);

  // The object chosen is read again for a listing read again, which may say something else of it.
  useEffect(() => {
    void spawnService.describe(selectedIndex);
  }, [selectedIndex, sessionId, spawnRevision, spawnService]);

  // The tree shows what is picked: an object picked elsewhere chosen, what stands above it opened and any filter
  // hiding it cleared; anything else picked, or nothing, chooses nothing here. The object shown, found again in a
  // listing read anew, is chosen where it stands now, the tree otherwise left as it is.
  useEffect(() => {
    // An object picked before the listing reaches the panel is chosen once it does.
    if (picked === heard.current || (picked?.kind === ELevelPick.SPAWN && !description)) {
      return;
    }

    heard.current = picked;

    if (picked && picked === shown.current) {
      return;
    }

    if (isPickRelocated) {
      if (shown.current) {
        shown.current = picked;
        select(picked?.kind === ELevelPick.SPAWN ? toLevelSpawnObjectId(picked.object.index) : null);
      }

      return;
    }

    if (picked?.kind === ELevelPick.SPAWN) {
      const { category, index, section } = picked.object;

      shown.current = picked;
      setFilter("");
      expandAll([toLevelSpawnCategoryId(category), toLevelSpawnSectionId(category, section)]);
      select(toLevelSpawnObjectId(index));
    } else {
      shown.current = null;
      select(null);
    }
  }, [description, expandAll, isPickRelocated, picked, select]);

  // A filter that matched inside a group opens it, because a closed group answering a query looks like no answer.
  useEffect(() => {
    if (filter.trim()) {
      expandAll(listLevelSpawnGroupIds(items));
    }
  }, [expandAll, filter, items]);

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
        label={toEmptyLabel(loadService.spawnReport)}
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
            details={spawnService.details.state}
            onGoTo={onGoToSelected}
          />
        </div>
      ) : null}
    </div>
  );
}
