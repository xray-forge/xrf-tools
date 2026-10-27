import { useInjection } from "@wirestate/react";
import { DependencyList, useEffect, useId, useMemo } from "react";

import { useCurrentTool } from "@/core/routing/current-tool.context";
import { EditorShellService } from "@/core/shell/services/editor-shell";

import { IEditorPanel } from "./editor-panel";

/** Returns panels whose owning tool matches the container currently rendering them. */
export function useEditorPanelsRegistry(): ReadonlyArray<IEditorPanel> {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const tool: string = useCurrentTool()?.path ?? "root";

  return editorShellService.getPanels(tool);
}

/**
 * Publishes panels while their owner is mounted.
 *
 * @param createPanels - Creates the current render closures.
 * @param dependencies - Values captured by the factory, checked at call sites by ESLint.
 */
export function useEditorPanels(createPanels: () => Array<IEditorPanel>, dependencies: DependencyList): void {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const owner: string = useId();
  const tool: string = useCurrentTool()?.path ?? "root";

  // Call sites are checked as dependency-aware hooks by ESLint.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const panels: Array<IEditorPanel> = useMemo(createPanels, dependencies);

  useEffect(() => {
    editorShellService.publishPanels(owner, tool, panels);
  }, [tool, owner, panels, editorShellService]);

  useEffect(() => () => editorShellService.releasePanels(owner), [tool, owner, editorShellService]);
}
