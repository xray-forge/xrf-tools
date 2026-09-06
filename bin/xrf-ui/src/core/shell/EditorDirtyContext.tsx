import { createContext, ReactElement, ReactNode, useCallback, useContext, useEffect, useMemo, useState } from "react";

import { UnsavedChangesDialog } from "@/core/ui/dialog/UnsavedChangesDialog";
import { Nullable } from "@/lib/types/general";

/**
 * Writes everything the active editor is holding.
 */
export type EditorSaver = () => Promise<boolean>;

interface IEditorDirtyContextValue {
  dirtyCount: number;
  save: Nullable<EditorSaver>;
  publish: (dirtyCount: number, save: Nullable<EditorSaver>) => void;
  requestLeave: (leave: () => void) => void;
}

const EditorDirtyContext = createContext<IEditorDirtyContextValue>({
  dirtyCount: 0,
  save: null,
  publish: () => {},
  requestLeave: (leave: () => void) => leave(),
});

/**
 * @returns How many of the active editor's files hold edits that are not on disk.
 */
export function useEditorDirtyCount(): number {
  return useContext(EditorDirtyContext).dirtyCount;
}

/**
 * Runs an action that would abandon the active editor, asking first when work would be lost.
 */
export function useRequestLeave(): (leave: () => void) => void {
  return useContext(EditorDirtyContext).requestLeave;
}

/**
 * Publishes how much unsaved work the active editor is holding, and what can write it.
 *
 * Call this from whatever owns the edited node rather than from a panel: a draft that stops being published because
 * somebody switched panels would be silently discardable, which is the state the prompt exists to prevent.
 *
 * @param dirtyCount - Number of files holding edits that have not been written.
 * @param save - Writes them, or null when this editor cannot - which makes the prompt offer only discarding. Memoize
 *   it, since it is published on every change of identity.
 */
export function useEditorDirty(dirtyCount: number, save: Nullable<EditorSaver> = null): void {
  const { publish } = useContext(EditorDirtyContext);

  useEffect(() => {
    publish(dirtyCount, save);

    return () => publish(0, null);
  }, [dirtyCount, save, publish]);
}

export function EditorDirtyProvider({ children }: { children: ReactNode }): ReactElement {
  const [dirtyCount, setDirtyCount] = useState<number>(0);
  const [save, setSave] = useState<Nullable<EditorSaver>>(null);
  const [pendingLeave, setPendingLeave] = useState<Nullable<() => void>>(null);
  const [isSaving, setSaving] = useState<boolean>(false);

  // Both stored as thunks, so `useState` invokes nothing while it holds a callback.
  const publish = useCallback((nextDirtyCount: number, nextSave: Nullable<EditorSaver>) => {
    setDirtyCount(nextDirtyCount);
    setSave(() => nextSave);
  }, []);

  const requestLeave = useCallback(
    (leave: () => void) => {
      if (dirtyCount > 0) {
        setPendingLeave(() => leave);
      } else {
        leave();
      }
    },
    [dirtyCount]
  );

  const value: IEditorDirtyContextValue = useMemo(
    () => ({ dirtyCount, publish, requestLeave, save }),
    [dirtyCount, publish, requestLeave, save]
  );

  const onLeave = useCallback(() => {
    pendingLeave?.();
    setPendingLeave(null);
  }, [pendingLeave]);

  const onClose = useCallback(() => {
    setPendingLeave(null);
  }, []);

  const onSave = useCallback(() => {
    if (!save) {
      return;
    }

    setSaving(true);

    void save()
      .then((isWritten: boolean) => {
        // A refused save keeps the dialog up rather than leaving anyway, so the work is still there to look at.
        if (isWritten) {
          onLeave();
        }
      })
      .finally(() => setSaving(false));
  }, [onLeave, save]);

  return (
    <EditorDirtyContext.Provider value={value}>
      {children}

      <UnsavedChangesDialog
        isOpen={pendingLeave !== null}
        isSaving={isSaving}
        description={
          `${dirtyCount} ${dirtyCount === 1 ? "file has" : "files have"} edits that are not written to disk. ` +
          "Leaving discards them."
        }
        onSave={save ? onSave : null}
        onDiscard={onLeave}
        onClose={onClose}
      />
    </EditorDirtyContext.Provider>
  );
}
