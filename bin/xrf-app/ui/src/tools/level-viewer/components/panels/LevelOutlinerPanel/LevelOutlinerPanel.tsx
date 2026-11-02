import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import {
  LEVEL_OBJECT_KINDS,
  LevelObject,
  LevelObjectKind,
  LevelObjectKindEntry,
  LevelObjectsDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Maybe, Nullable } from "@xrf/types";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EmptyListing } from "@xrf/ui/layout/empty-listing";
import { Chip } from "@xrf/ui/material/chip";
import { cn } from "@xrf/ui/theme";
import {
  ITreeNode,
  ITreeSelectGesture,
  IUseTreeState,
  TreeRowLabel,
  useTreeState,
  VirtualizedTree,
} from "@xrf/ui/tree";
import { ReactElement, ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { LevelKindIcon } from "@/core/level/components/objects/LevelKindIcon";
import { isLevelHelperOn } from "@/core/level/lib/objects/level-helper-options";
import { toLevelObjectKey } from "@/core/level/lib/objects/level-object-kinds";
import { ILevelVisibleOptionsChange, TLevelVisibleOption } from "@/core/level/lib/objects/level-object-visibility";
import { ELevelPick, TLevelPick, toLevelObjectPick } from "@/core/level/lib/pick/level-pick";
import { ILevelSpawnReport, isLevelSpawnReading } from "@/core/level/lib/spawn/level-spawn-report";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelViewService } from "@/core/level/services/level-view";
import { ELevelSelectionMode, LevelViewportService } from "@/core/level/services/level-viewport";
import { EditorSearchHeader } from "@/core/shell/chrome/EditorSearchHeader";
import {
  ILevelOutlinerFilter,
  listLevelOutlinerGroupIds,
  listLevelOutlinerObjects,
  listLevelOutlinerParentIds,
  TLevelOutlinerRow,
  toLevelOutlinerObjectId,
  toLevelOutlinerTree,
} from "@/tools/level-viewer/lib/outliner/level-outliner-tree";
import { describeLevelSpawnRelease } from "@/tools/level-viewer/lib/spawn/level-spawn-release";
import { LevelViewerObjectsService } from "@/tools/level-viewer/services/level-viewer-objects";

import { LevelOutlinerKindActions } from "./LevelOutlinerKindActions";
import { LevelOutlinerObjectActions } from "./LevelOutlinerObjectActions";

/**
 * What the panel says while it lists nothing: why the spawn could not be read, that it is being read, or that it
 * places nothing.
 */
function toEmptyLabel(report: ILevelSpawnReport): string {
  if (report.failure) {
    return `The level's spawn could not be read: ${report.failure}`;
  }

  return !report.isListed || isLevelSpawnReading(report)
    ? "Reading the level's spawn."
    : "The level's spawn places nothing.";
}

/** How many objects a row stands for: a group's count, or one. */
function toCount(row: Maybe<TLevelOutlinerRow>): number {
  return row && row.kind !== "object" ? row.count : 1;
}

/**
 * Every object placed in the level, `Group -> kind -> object`, a spawned kind's by section between: found by name,
 * section or visual and by kind, shown, hidden and helped a kind at a time, hidden one by one, selected as the viewport
 * selects, and gone to.
 */
export function LevelOutlinerPanel({
  "data-testid": dataTestId = "level-outliner-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewService: LevelViewService = useInjection(LevelViewService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const objectsService: LevelViewerObjectsService = useInjection(LevelViewerObjectsService);

  const tree: IUseTreeState = useTreeState();
  const { expandAll, select } = tree;
  const [text, setText] = useState<string>("");
  const [kinds, setKinds] = useState<ReadonlySet<LevelObjectKind>>(new Set());
  // Read here rather than in the rows' callbacks, so a switch from the toolbar redraws the rows' toggles.
  const { options, helpers } = viewService;
  const { hiddenKeys, isPickRelocated, picked, selected } = viewportService;

  // A spawn placing nothing reads as one not listed yet would: the empty state says which.
  const description: Nullable<LevelObjectsDescription> = loadService.objects?.objects.length
    ? loadService.objects
    : null;
  const filter: ILevelOutlinerFilter = useMemo(() => ({ kinds, text }), [kinds, text]);
  const items: Array<ITreeNode<TLevelOutlinerRow>> = useMemo(
    () => (description ? toLevelOutlinerTree(description, filter) : []),
    [description, filter]
  );
  const listed: Array<LevelObject> = useMemo(() => listLevelOutlinerObjects(items), [items]);
  // The kinds the level holds, in the kind table's order: a chip for a kind it has none of would filter to nothing.
  const present: Array<LevelObjectKindEntry> = useMemo(() => {
    const held: ReadonlySet<LevelObjectKind> = new Set(description?.objects.map((it: LevelObject) => it.kind));

    return LEVEL_OBJECT_KINDS.filter((entry: LevelObjectKindEntry) => held.has(entry.kind));
  }, [description]);
  const matched: number = useMemo(
    () => items.reduce((sum: number, it: ITreeNode<TLevelOutlinerRow>) => sum + toCount(it.payload), 0),
    [items]
  );
  const markedIds: ReadonlySet<string> = useMemo(() => new Set(selected.map(toLevelOutlinerObjectId)), [selected]);

  // The pick last heard, so the tree answers each once.
  const heard = useRef<Nullable<TLevelPick>>(null);
  // The pick the rows chose last, so the tree leaves itself as it is when it hears its own choice back.
  const shown = useRef<Nullable<TLevelPick>>(null);
  // The object a range runs from: the last one chosen without Shift.
  const anchor = useRef<Nullable<LevelObject>>(null);

  const sessionId: Nullable<string> = loadService.level.value?.selected.sessionId ?? null;

  const onSelect = useCallback(
    (item: ITreeNode<TLevelOutlinerRow>, gesture?: ITreeSelectGesture) => {
      select(item.id);

      if (item.payload?.kind !== "object" || !description) {
        return;
      }

      const { object } = item.payload;
      const pick: TLevelPick = toLevelObjectPick(object, description);

      shown.current = pick;

      if (gesture?.isExtending && anchor.current) {
        const from: number = listed.indexOf(anchor.current);
        const to: number = listed.indexOf(object);

        if (from !== -1 && to !== -1) {
          viewportService.noteSelected(listed.slice(Math.min(from, to), Math.max(from, to) + 1), pick);

          return;
        }
      }

      anchor.current = object;
      viewportService.notePicked(pick, gesture?.isToggling ? ELevelSelectionMode.TOGGLE : ELevelSelectionMode.REPLACE);
    },
    [description, listed, select, viewportService]
  );

  const onActivate = useCallback(
    (item: ITreeNode<TLevelOutlinerRow>) => {
      if (item.payload?.kind === "object") {
        void objectsService.goTo([item.payload.object]);
      }
    },
    [objectsService]
  );

  const onToggleKind = useCallback((kind: LevelObjectKind) => {
    setKinds((current: ReadonlySet<LevelObjectKind>) => {
      const next: Set<LevelObjectKind> = new Set(current);

      if (!next.delete(kind)) {
        next.add(kind);
      }

      return next;
    });
  }, []);

  const renderLabel = useCallback((item: ITreeNode<TLevelOutlinerRow>): ReactNode => {
    const row: Maybe<TLevelOutlinerRow> = item.payload;

    if (!row) {
      return item.label;
    }

    if (row.kind === "object") {
      const release = row.object.spawn?.release ?? null;

      return release ? (
        <span className={"opacity-55"} title={describeLevelSpawnRelease(release)}>
          {item.label}
        </span>
      ) : (
        item.label
      );
    }

    const label: ReactElement = (
      <TreeRowLabel
        label={item.label}
        caption={
          row.released ? `${formatCount(row.count)} · ${formatCount(row.released)} released` : formatCount(row.count)
        }
        captionTitle={
          row.released
            ? `${formatCount(row.count)} placed, ${formatCount(row.released)} of them released by a new game`
            : `${formatCount(row.count)} placed`
        }
      />
    );

    // A kind's row shows the icon its helpers draw, where they draw one.
    return row.kind === "kind" ? (
      <span className={"flex min-w-0 items-center gap-1"}>
        <LevelKindIcon kind={row.entry.kind} />
        {label}
      </span>
    ) : (
      label
    );
  }, []);

  const renderActions = useCallback(
    (item: ITreeNode<TLevelOutlinerRow>): ReactNode => {
      const row: Maybe<TLevelOutlinerRow> = item.payload;

      if (!row || row.kind === "section") {
        return null;
      }

      if (row.kind === "object") {
        return (
          <LevelOutlinerObjectActions
            name={row.object.name}
            isHidden={hiddenKeys.has(toLevelObjectKey(row.object.id))}
            onToggle={() => viewportService.toggleHidden(row.object)}
          />
        );
      }

      return (
        <LevelOutlinerKindActions
          label={item.label}
          kinds={row.kind === "group" ? row.kinds : [row.entry]}
          isVisible={(option: TLevelVisibleOption) => options[option]}
          isHelped={(kind: LevelObjectKind) => isLevelHelperOn(helpers, kind)}
          isHelpersShown={helpers.isShown}
          onVisibleChange={(change: ILevelVisibleOptionsChange) =>
            viewService.setOptions({
              ...options,
              ...Object.fromEntries(change.options.map((option: TLevelVisibleOption) => [option, change.isOn])),
            })
          }
          onHelperChange={viewService.setHelperKinds}
        />
      );
    },
    [helpers, hiddenKeys, options, viewService, viewportService]
  );

  // What was chosen belongs to the level it was chosen in.
  useEffect(() => {
    shown.current = null;
    anchor.current = null;
    select(null);
  }, [sessionId, select]);

  // The tree shows what is picked: an object picked elsewhere chosen, what stands above it opened and any filter
  // hiding it cleared; anything else picked, or nothing, chooses nothing here. The object shown, found again in a
  // listing read anew, is chosen where it stands now, the tree otherwise left as it is.
  useEffect(() => {
    // An object picked before the listing reaches the panel is chosen once it does.
    if (picked === heard.current || (picked?.kind === ELevelPick.OBJECT && !description)) {
      return;
    }

    heard.current = picked;

    if (picked && picked === shown.current) {
      return;
    }

    if (isPickRelocated) {
      if (shown.current) {
        shown.current = picked;
        select(picked?.kind === ELevelPick.OBJECT ? toLevelOutlinerObjectId(picked.object) : null);
      }

      return;
    }

    if (picked?.kind === ELevelPick.OBJECT) {
      shown.current = picked;
      anchor.current = picked.object;
      setText("");
      setKinds(new Set());
      expandAll(listLevelOutlinerParentIds(picked.object));
      select(toLevelOutlinerObjectId(picked.object));
    } else {
      shown.current = null;
      select(null);
    }
  }, [description, expandAll, isPickRelocated, picked, select]);

  // A filter that matched inside a group opens it, because a closed group answering a query looks like no answer.
  useEffect(() => {
    if (text.trim() || kinds.size) {
      expandAll(listLevelOutlinerGroupIds(items));
    }
  }, [expandAll, items, kinds, text]);

  if (!loadService.level.value) {
    return (
      <EditorPanelEmpty
        data-testid={dataTestId}
        id={id}
        className={className}
        label={"No level open. Open one to list what is placed in it."}
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
        title={"Outliner"}
        count={matched}
        query={text}
        placeholder={"Filter by name, section or visual"}
        ariaLabel={"Filter level objects"}
        onClear={() => setText("")}
        onQueryChange={setText}
      />

      <div className={"flex shrink-0 flex-wrap gap-1 border-b border-divider px-2 py-1"} role={"group"}>
        {present.map((entry: LevelObjectKindEntry) => (
          <Chip
            key={entry.kind}
            data-testid={"level-outliner-kind-chip"}
            size={"small"}
            label={entry.label}
            variant={kinds.has(entry.kind) ? "filled" : "outlined"}
            aria-pressed={kinds.has(entry.kind)}
            onClick={() => onToggleKind(entry.kind)}
          />
        ))}
      </div>

      {items.length ? (
        <div className={"min-h-0 grow"}>
          <VirtualizedTree<TLevelOutlinerRow>
            ariaLabel={"Level objects"}
            className={"h-full"}
            items={items}
            expandedIds={tree.expandedIds}
            selectedId={tree.selectedId}
            markedIds={markedIds}
            renderLabel={renderLabel}
            renderActions={renderActions}
            onToggleExpanded={tree.toggleExpanded}
            onSelect={onSelect}
            onActivate={onActivate}
          />
        </div>
      ) : (
        <EmptyListing label={"No level object matches that."} />
      )}
    </div>
  );
}
