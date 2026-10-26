import { useInjection } from "@wirestate/react";
import { DependencyList, useEffect, useId, useMemo } from "react";

import { useCurrentApplication } from "@/core/routing/current-application.context";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { EditorShellService } from "@/core/shell/services/editor-shell";

/** Returns menu entries published by the application currently rendering, beside whatever else publishes. */
export function useEditorMenusRegistry(): ReadonlyArray<IMenuContribution> {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const application: string = useCurrentApplication()?.path ?? "root";

  return editorShellService.getMenus(application);
}

/**
 * Publishes menu entries while their owner is mounted. Many owners may publish into one section at once.
 *
 * @param createContributions - Creates the current entries, grouped by the section each goes into.
 * @param dependencies - Values captured by the factory, checked at call sites by ESLint.
 */
export function useEditorMenus(
  createContributions: () => Array<IMenuContribution>,
  dependencies: DependencyList
): void {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const owner: string = useId();
  const application: string = useCurrentApplication()?.path ?? "root";

  // Call sites are checked as dependency-aware hooks by ESLint.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const contributions: Array<IMenuContribution> = useMemo(createContributions, dependencies);

  useEffect(() => {
    editorShellService.publishMenus(owner, application, contributions);
  }, [application, owner, contributions, editorShellService]);

  useEffect(() => () => editorShellService.releaseMenus(owner), [editorShellService, owner]);
}
