import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import {
  SceneBinaryToolsDescription,
  SceneObjectId,
  SceneObjectsDescription,
  SelectedSceneDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Maybe, Nullable } from "@xrf/types";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EmptyListing } from "@xrf/ui/layout/empty-listing";
import { cn } from "@xrf/ui/theme";
import { ITreeNode, IUseTreeState, TreeRowLabel, useTreeState, VirtualizedTree } from "@xrf/ui/tree";
import { ReactElement, ReactNode, useCallback, useEffect, useMemo, useState } from "react";

import { EditorSearchHeader } from "@/core/shell/chrome/EditorSearchHeader";
import { noop } from "@/lib/callbacks/noop";
import {
  countLevelEditorOutlinerObjects,
  ELevelEditorOutlinerRow,
  ILevelEditorOutlinerFilter,
  ILevelEditorOutlinerSources,
  listLevelEditorOutlinerParentIds,
  TLevelEditorOutlinerRow,
  toLevelEditorOutlinerObjectId,
  toLevelEditorOutlinerTree,
} from "@/tools/level-editor/lib/outliner/level-editor-outliner-tree";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

import { LevelEditorOutlinerToolLabel } from "./LevelEditorOutlinerToolLabel";

/** What the panel says while the scene's objects are not listed: why they could not be read, or that they are read. */
function toEmptyLabel(objects: AsyncState<SceneObjectsDescription>): string {
  return objects.error
    ? `The scene's objects could not be read: ${objects.error.message}`
    : "Reading the scene's objects.";
}

/**
 * Everything the open scene holds, `tool -> object`: each tool in the SDK's order with what it holds, a binary tool
 * counted, a tool with problems marked; found by name, reference or section, and selected for the Inspector.
 */
export function LevelEditorOutlinerPanel({
  "data-testid": dataTestId = "level-editor-outliner-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);
  const selectionService: LevelEditorSelectionService = useInjection(LevelEditorSelectionService);

  const tree: IUseTreeState = useTreeState();
  const { expandAll, select } = tree;
  const [text, setText] = useState<string>("");

  const scene: Nullable<SelectedSceneDescription> = sceneService.scene.value;
  const sessionId: Nullable<string> = sceneService.scene.sessionId;
  const objects: Nullable<SceneObjectsDescription> = sceneService.objects.value;
  const binaryTools: Nullable<SceneBinaryToolsDescription> = sceneService.binaryTools.value;
  const selectedId: Nullable<SceneObjectId> = selectionService.selectedId;

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
  const activeId: Nullable<string> = selectedId ? toLevelEditorOutlinerObjectId(selectedId) : null;

  const onSelect = useCallback(
    (item: ITreeNode<TLevelEditorOutlinerRow>) => {
      select(item.id);

      if (item.payload?.kind === ELevelEditorOutlinerRow.OBJECT) {
        selectionService.select(item.payload.object.id);
      }
    },
    [select, selectionService]
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

  // What the tree's cursor stands on belongs to the scene it was chosen in.
  useEffect(() => {
    select(null);
  }, [sessionId, select]);

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
            onToggleExpanded={tree.toggleExpanded}
            onSelect={onSelect}
            onActivate={noop}
          />
        </div>
      ) : (
        <EmptyListing label={"No scene object matches that."} />
      )}
    </div>
  );
}
