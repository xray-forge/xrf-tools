import { Nullable } from "@xrf/types";
import { Dispatch, SetStateAction, useCallback, useMemo, useState } from "react";

export interface IArchiveRunConfirmation<TRun extends string> {
  /** Which run the confirmation is for, or null while it is closed. */
  run: Nullable<TRun>;
  /** Whether the user agreed to replace the volumes the destination already holds. */
  isForced: boolean;
  setIsForced: Dispatch<SetStateAction<boolean>>;
  /** Opens the confirmation for one run, asking about replacing volumes afresh. */
  open: (run: TRun) => void;
  close: () => void;
}

/**
 * The confirmation an archive configuration editor asks before a run. Agreeing to replace volumes holds for one
 * confirmation only, so every opening asks again.
 */
export function useArchiveRunConfirmation<TRun extends string>(): IArchiveRunConfirmation<TRun> {
  const [run, setRun] = useState<Nullable<TRun>>(null);
  const [isForced, setIsForced] = useState<boolean>(false);

  const open = useCallback((next: TRun) => {
    setIsForced(false);
    setRun(next);
  }, []);

  const close = useCallback(() => setRun(null), []);

  return useMemo(() => ({ run, isForced, setIsForced, open, close }), [run, isForced, open, close]);
}
