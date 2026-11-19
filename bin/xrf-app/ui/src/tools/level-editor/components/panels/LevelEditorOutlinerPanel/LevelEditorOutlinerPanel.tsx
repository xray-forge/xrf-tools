import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { formatCount } from "@xrf/format";
import {
  SceneBinaryToolsDescription,
  SceneObjectId,
  SceneObjectsDescription,
  SelectedSceneDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Maybe, Nullable } from "@xrf/types";
import { Button } from "@xrf/ui/button";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EmptyListing } from "@xrf/ui/layout/empty-listing";
import { cn } from "@xrf/ui/theme";
import { ITreeNode, IUseTreeState, TreeRowLabel, useTreeState, VirtualizedTree } from "@xrf/ui/tree";
import { ReactElement, ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { LevelOutlinerObjectActions } from "@/core/level/components/objects/LevelOutlinerObjectActions";
import { EditorSearchHeader } from "@/core/shell/chrome/EditorSearchHeader";
import {
  countLevelEditorOutlinerObjects,
  ELevelEditorOutlinerRow,
  ILevelEditorOutlinerFilter,
  ILevelEditorOutlinerSources,
  listLevelEditorOutlinerParentIds,
  TLevelEditorOutlinerRow,
  toLevelEditorOutlinerObjectId,
  toLevelEditorOutlinerToolId,
  toLevelEditorOutlinerTree,
} from "@/tools/level-editor/lib/outliner/level-editor-outliner-tree";
import { LevelEditorObjectsService } from "@/tools/level-editor/services/level-editor-objects";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

import { LevelEditorOutlinerToolLabel } from "./LevelEditorOutlinerToolLabel";

/** What the panel says while the scene's objects are not listed: the read error, or that they are being read. */
function toEmptyLabel(objects: AsyncState<SceneObjectsDescription>): string {
  return objects.error
    ? `The scene's objects could not be read: ${objects.error.message}`
    : "Reading the scene's objects.";
}

/**
 * The open scene as a `tool -> object` tree, in the SDK's tool order, filterable by name, reference or section.
 *
 * Its selection is shared with the viewport and the Inspector; activating a row frames it, and each row can be hidden.
 */
export function LevelEditorOutlinerPanel({
  "data-testid": dataTestId = "level-editor-outliner-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);
  const selectionService: LevelEditorSelectionService = useInjection(LevelEditorSelectionService);
  const objectsService: LevelEditorObjectsService = useInjection(LevelEditorObjectsService);

  const tree: IUseTreeState = useTreeState();
  const { expandAll, select } = tree;
  const [text, setText] = useState<string>("");

  const scene: Nullable<SelectedSceneDescription> = sceneService.scene.value;
  const sessionId: Nullable<string> = sceneService.scene.sessionId;
  const objects: Nullable<SceneObjectsDescription> = sceneService.objects.value;
  const binaryTools: Nullable<SceneBinaryToolsDescription> = sceneService.binaryTools.value;
  const selectedId: Nullable<SceneObjectId> = selectionService.selectedId;
  // Read here rather than in the rows' callbacks, so hiding one redraws the rows' eyes.
  const hidden: ReadonlyArray<SceneObjectId> = selectionService.hidden;

  const sources: ILevelEditorOutlinerSources = useMemo(
    () => ({
      binaryTools,
      objects,
      problems: [...(scene?.problems ?? []), ...(objects?.problems ?? []), ...(binaryTools?.problems ?? [])],
      tools: scene?.tools ?? [],
    }),
    [binaryTools, objects, scene]
  );
  const filter: ILevelEditorOutlinerFilter = useMemo(() => ({ text }), [text]);
  const items: Array<ITreeNode<TLevelEditorOutlinerRow>> = useMemo(
    () => toLevelEditorOutlinerTree(sources, filter),
    [filter, sources]
  );
  const matched: number = useMemo(() => countLevelEditorOutlinerObjects(items), [items]);
  const hiddenRowIds: ReadonlySet<string> = useMemo(() => new Set(hidden.map(toLevelEditorOutlinerObjectId)), [hidden]);
  const activeId: Nullable<string> = selectedId ? toLevelEditorOutlinerObjectId(selectedId) : null;

  // The row the tree chose last, so it ignores its own choice echoed back from the selection.
  const chosen = useRef<Nullable<string>>(null);

  const onSelect = useCallback(
    (item: ITreeNode<TLevelEditorOutlinerRow>) => {
      select(item.id);

      if (item.payload?.kind === ELevelEditorOutlinerRow.OBJECT) {
        chosen.current = item.id;
        selectionService.select(item.payload.object.id);
      }
    },
    [select, selectionService]
  );

  const onActivate = useCallback(
    (item: ITreeNode<TLevelEditorOutlinerRow>) => {
      if (item.payload?.kind === ELevelEditorOutlinerRow.OBJECT) {
        chosen.current = item.id;
        objectsService.focus(item.payload.object.id);
      }
    },
    [objectsService]
  );

  const onClear = useCallback(() => setText(""), []);

  const renderLabel = useCallback((item: ITreeNode<TLevelEditorOutlinerRow>): ReactNode => {
    const row: Maybe<TLevelEditorOutlinerRow> = item.payload;

    if (!row) {
      return item.label;
    }

    if (row.kind === ELevelEditorOutlinerRow.TOOL) {
      return <LevelEditorOutlinerToolLabel label={item.label} caption={row.caption} problems={row.problems} />;
    }

    return row.object.group ? (
      <TreeRowLabel label={item.label} caption={row.object.group} captionTitle={`Member of ${row.object.group}`} />
    ) : (
      item.label
    );
  }, []);

  const renderActions = useCallback(
    (item: ITreeNode<TLevelEditorOutlinerRow>): ReactNode => {
      const row: Maybe<TLevelEditorOutlinerRow> = item.payload;

      return row?.kind === ELevelEditorOutlinerRow.OBJECT ? (
        <LevelOutlinerObjectActions
          name={row.object.name}
          isHidden={hiddenRowIds.has(item.id)}
          onToggle={() => selectionService.toggleHidden(row.object.id)}
        />
      ) : null;
    },
    [hiddenRowIds, selectionService]
  );

  // The tree's cursor belongs to the scene it was chosen in.
  useEffect(() => {
    chosen.current = null;
    select(null);
  }, [sessionId, select]);

  // A selection made elsewhere, such as a viewport pick, clears the filter and opens its tool to reveal the row.
  useEffect(() => {
    const rowId: Nullable<string> = selectedId ? toLevelEditorOutlinerObjectId(selectedId) : null;

    if (rowId === chosen.current) {
      return;
    }

    chosen.current = rowId;

    if (selectedId && rowId) {
      setText("");
      expandAll([toLevelEditorOutlinerToolId(selectedId.tool)]);
      select(rowId);
    } else {
      select(null);
    }
  }, [expandAll, select, selectedId]);

  // A filter that matched inside a tool opens it, because a closed tool answering a query looks like no answer.
  useEffect(() => {
    if (text.trim()) {
      expandAll(listLevelEditorOutlinerParentIds(items));
    }
  }, [expandAll, items, text]);

  if (!scene) {
    return (
      <EditorPanelEmpty
        data-testid={dataTestId}
        id={id}
        className={className}
        label={"No scene open. Open one to list what it holds."}
      />
    );
  }

  if (!objects) {
    return (
      <EditorPanelEmpty
        data-testid={dataTestId}
        id={id}
        className={className}
        label={toEmptyLabel(sceneService.objects.state)}
      />
    );
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex h-full min-h-0 flex-col", className)}>
      <EditorSearchHeader
        title={"Outliner"}
        count={matched}
        query={text}
        placeholder={"Filter by name, reference or section"}
        ariaLabel={"Filter scene objects"}
        onClear={onClear}
        onQueryChange={setText}
      />

      {hidden.length ? (
        <div
          className={"flex shrink-0 items-center justify-between gap-2 border-b border-divider px-2 py-1 text-xs"}
          role={"status"}
        >
          <span>{`${formatCount(hidden.length)} hidden`}</span>
          <Button size={"small"} variant={"text"} onClick={selectionService.revealAll}>
            Reveal hidden objects
          </Button>
        </div>
      ) : null}

      {items.length ? (
        <div className={"min-h-0 grow"}>
          <VirtualizedTree<TLevelEditorOutlinerRow>
            ariaLabel={"Scene outline"}
            className={"h-full"}
            items={items}
            expandedIds={tree.expandedIds}
            selectedId={tree.selectedId}
            activeId={activeId}
            renderLabel={renderLabel}
            renderActions={renderActions}
            onToggleExpanded={tree.toggleExpanded}
            onSelect={onSelect}
            onActivate={onActivate}
          />
        </div>
      ) : (
        <EmptyListing label={"No scene object matches that."} />
      )}
    </div>
  );
}
