import { useInjection } from "@wirestate/react";
import { useEffect, useId } from "react";

import { useCurrentTool } from "@/core/routing/current-tool.context";
import { TEditorStatusSegment } from "@/core/shell/editor-shell/editor-status-segment";
import { EditorShellService } from "@/core/shell/services/editor-shell";

/** Returns status for the active tool, excluding an outgoing editor's publication. */
export function useEditorStatusSegments(): ReadonlyArray<TEditorStatusSegment> {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const tool: string = useCurrentTool()?.path ?? "root";

  return editorShellService.getStatus(tool);
}

/** Publishes status while its owner is mounted. Equal segments keep the existing service snapshot. */
export function useEditorStatus(segments: ReadonlyArray<TEditorStatusSegment>): void {
  const editorShellService: EditorShellService = useInjection(EditorShellService);
  const owner: string = useId();
  const tool: string = useCurrentTool()?.path ?? "root";

  useEffect(() => {
    editorShellService.publishStatus(owner, tool, segments);
  }, [tool, owner, segments, editorShellService]);

  useEffect(() => () => editorShellService.releaseStatus(owner), [tool, owner, editorShellService]);
}
