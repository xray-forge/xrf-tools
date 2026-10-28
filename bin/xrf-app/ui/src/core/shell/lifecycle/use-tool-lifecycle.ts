import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { useEffect, useId } from "react";

import { ToolLifecycleService, TToolSaver } from "@/core/shell/services/tool-lifecycle";

export interface IToolLifecycle {
  isBusy: boolean;
  dirtyCount: number;
  /** Saves all pending edits; false keeps the tool open. Null offers discard and stay only. */
  save: Nullable<TToolSaver>;
}

/** Publishes the active document's lifecycle from its owner, independently of visible panels. */
export function useToolLifecycle({ isBusy, dirtyCount, save }: IToolLifecycle): void {
  useToolBusy(isBusy);
  useToolDirty(dirtyCount, save);
}

/** Registers one navigation-blocking operation until its owner unmounts. */
export function useToolBusy(isBusy: boolean): void {
  const lifecycle: ToolLifecycleService = useInjection(ToolLifecycleService);
  const owner: string = useId();

  useEffect(() => {
    lifecycle.setBusy(owner, isBusy);

    return () => lifecycle.setBusy(owner, false);
  }, [isBusy, lifecycle, owner]);
}

/** Publishes the active document's edits and saver; panels must not own this registration. */
export function useToolDirty(dirtyCount: number, save: Nullable<TToolSaver> = null): void {
  const lifecycle: ToolLifecycleService = useInjection(ToolLifecycleService);
  const owner: string = useId();

  useEffect(() => {
    lifecycle.setDraft(owner, dirtyCount, save);
  }, [dirtyCount, lifecycle, owner, save]);

  useEffect(() => () => lifecycle.releaseDraft(owner), [lifecycle, owner]);
}

/** Returns whether any tool operation or save-and-leave operation blocks navigation. */
export function useIsToolBusy(): boolean {
  return useInjection(ToolLifecycleService).isBusy;
}

/** Returns the shared leave policy for an action that abandons the active document. */
export function useRequestLeave(): (leave: () => void) => void {
  return useInjection(ToolLifecycleService).requestLeave;
}
