import { useInjection } from "@wirestate/react";
import { useEffect, useId } from "react";

import { useCurrentApplication } from "@/core/routing/current-application.context";
import { TEditorStatusSegment } from "@/core/shell/editor-shell/editor-status-segment";
import { EditorShellService } from "@/core/shell/services/editor-shell";

/** Returns status for the active application, excluding an outgoing editor's publication. */
export function useEditorStatusSegments(): ReadonlyArray<TEditorStatusSegment> {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const application: string = useCurrentApplication()?.path ?? "root";

  return editorShellService.getStatus(application);
}

/** Publishes status while its owner is mounted. Equal segments keep the existing service snapshot. */
export function useEditorStatus(segments: ReadonlyArray<TEditorStatusSegment>): void {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const owner: string = useId();
  const application: string = useCurrentApplication()?.path ?? "root";

  useEffect(() => {
    editorShellService.publishStatus(owner, application, segments);
  }, [application, owner, segments, editorShellService]);

  useEffect(() => () => editorShellService.releaseStatus(owner), [application, owner, editorShellService]);
}
