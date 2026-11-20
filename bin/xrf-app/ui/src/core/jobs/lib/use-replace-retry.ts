import { Nullable } from "@xrf/types";
import { useCallback, useMemo, useState } from "react";

import { TCommandCompletion } from "@/core/operations/lib/command-reporter";

import { isReplaceRefusal } from "./replace-refusal";

/** One start of a run that refuses, unforced, to replace files already there. */
export interface IReplaceRetryStart<T> {
  request: T;
  /** Whether the run may replace them, which it is only once the person agreed to. */
  isForced: boolean;
}

/** A run started unforced, held while the page asks whether to replace what it refused over. */
export interface IReplaceRetry<T> {
  /** Why the held run was refused, for the page to ask about; null while none is held. */
  refusal: Nullable<string>;
  /** Runs the request unforced, holding it where it refused only because files it writes are already there. */
  run: (request: T) => Promise<void>;
  /** Runs the held request again, forced. */
  replace: () => Promise<void>;
  /** Lets the held request go, replacing nothing. */
  dismiss: () => void;
}

/**
 * @param start - Starts the run, answering how it ended, or null where it did not start.
 * @returns The run, and the refusal the page asks about.
 */
export function useReplaceRetry<T>(
  start: (run: IReplaceRetryStart<T>) => Promise<Nullable<TCommandCompletion<unknown>>>
): IReplaceRetry<T> {
  const [held, setHeld] = useState<Nullable<{ request: T; refusal: string }>>(null);

  const run = useCallback(
    async (request: T) => {
      const completion: Nullable<TCommandCompletion<unknown>> = await start({ request, isForced: false });
      const error: Nullable<Error> = completion?.error ?? null;

      if (error !== null && isReplaceRefusal(error)) {
        setHeld({ request, refusal: error.message });
      }
    },
    [start]
  );

  const replace = useCallback(async () => {
    if (held === null) {
      return;
    }

    setHeld(null);

    await start({ request: held.request, isForced: true });
  }, [held, start]);

  const dismiss = useCallback(() => setHeld(null), []);

  const refusal: Nullable<string> = held?.refusal ?? null;

  return useMemo(() => ({ refusal, run, replace, dismiss }), [dismiss, refusal, replace, run]);
}
