import { useInjection } from "@wirestate/react";
import { useEffect, useId } from "react";

import { EditorLifecycleService, EditorSaver } from "@/core/shell/services/editor-lifecycle";
import { Nullable } from "@/lib/types/general";

export interface IEditorLifecycle {
  isBusy: boolean;
  dirtyCount: number;
  /** Saves all pending edits; false keeps the editor open. Null offers discard and stay only. */
  save: Nullable<EditorSaver>;
}

/** Publishes the active document's lifecycle from its owner, independently of visible panels. */
export function useEditorLifecycle({ isBusy, dirtyCount, save }: IEditorLifecycle): void {
  useEditorBusy(isBusy);
  useEditorDirty(dirtyCount, save);
}

/** Registers one navigation-blocking operation until its owner unmounts. */
export function useEditorBusy(isBusy: boolean): void {
  const lifecycle: EditorLifecycleService = useInjection(EditorLifecycleService);
  const owner: string = useId();

  useEffect(() => {
    lifecycle.setBusy(owner, isBusy);

    return () => lifecycle.setBusy(owner, false);
  }, [isBusy, lifecycle, owner]);
}

/** Publishes the active document's edits and saver; panels must not own this registration. */
export function useEditorDirty(dirtyCount: number, save: Nullable<EditorSaver> = null): void {
  const lifecycle: EditorLifecycleService = useInjection(EditorLifecycleService);
  const owner: string = useId();

  useEffect(() => {
    lifecycle.setDraft(owner, dirtyCount, save);
  }, [dirtyCount, lifecycle, owner, save]);

  useEffect(() => () => lifecycle.releaseDraft(owner), [lifecycle, owner]);
}

/** Returns whether any editor operation or save-and-leave operation blocks navigation. */
export function useIsEditorBusy(): boolean {
  return useInjection(EditorLifecycleService).isBusy;
}

/** Returns the active document's count of files with unsaved edits. */
export function useEditorDirtyCount(): number {
  return useInjection(EditorLifecycleService).dirtyCount;
}

/** Returns the shared leave policy for an action that abandons the active document. */
export function useRequestLeave(): (leave: () => void) => void {
  return useInjection(EditorLifecycleService).requestLeave;
}
