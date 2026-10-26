import { useInjection } from "@wirestate/react";
import { DependencyList, useEffect, useId, useMemo } from "react";

import { useCurrentApplication } from "@/core/routing/current-application.context";
import { EditorShellService } from "@/core/shell/services/editor-shell";

import { IEditorPanel } from "./editor-panel";

/** Returns panels whose owning application matches the container currently rendering them. */
export function useEditorPanelsRegistry(): ReadonlyArray<IEditorPanel> {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const application: string = useCurrentApplication()?.path ?? "root";

  return editorShellService.getPanels(application);
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
  const application: string = useCurrentApplication()?.path ?? "root";

  // Call sites are checked as dependency-aware hooks by ESLint.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const panels: Array<IEditorPanel> = useMemo(createPanels, dependencies);

  useEffect(() => {
    editorShellService.publishPanels(owner, application, panels);
  }, [application, owner, panels, editorShellService]);

  useEffect(() => () => editorShellService.releasePanels(owner), [application, owner, editorShellService]);
}
