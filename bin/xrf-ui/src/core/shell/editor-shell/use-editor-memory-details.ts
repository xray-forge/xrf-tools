import { useInjection } from "@wirestate/react";
import { useEffect, useId } from "react";

import { TMemoryDetailSource } from "@/core/diagnostics/lib/memory-detail-source";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { EditorShellService } from "@/core/shell/services/editor-shell";

/** Returns the memory hover rows the active application adds, excluding an outgoing editor's. */
export function useEditorMemoryDetails(): ReadonlyArray<TMemoryDetailSource> {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const application: string = useCurrentApplication()?.path ?? "root";

  return editorShellService.getMemoryDetails(application);
}

/**
 * Adds rows to the status bar's memory hover while its owner is mounted, after the processes.
 *
 * @param sources - The rows, each read as the memory is; kept stable, since a new list publishes again.
 */
export function usePublishedMemoryDetails(sources: ReadonlyArray<TMemoryDetailSource>): void {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const owner: string = useId();
  const application: string = useCurrentApplication()?.path ?? "root";

  useEffect(() => {
    editorShellService.publishMemoryDetails(owner, application, sources);
  }, [application, owner, sources, editorShellService]);

  useEffect(() => () => editorShellService.releaseMemoryDetails(owner), [application, owner, editorShellService]);
}
