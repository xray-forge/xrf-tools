import { createContext, useCallback, useContext, useEffect, useMemo, useRef } from "react";

import { Nullable } from "@/lib/types/general";

/** What a form offers its fields: a way to be told when the form was actually run. */
export interface IFormCommitRegistry {
  /** Joins the form. Returns leaving it. */
  join: (commit: () => void) => () => void;
  /** Tells every field that joined that the form was submitted. */
  commit: () => void;
}

/**
 * The enclosing form, or `null` for a field that is not in one.
 *
 * Absent rather than required, because a path field is also used in side panels and toolbars where there is no
 * submission to speak of.
 */
export const FormCommitContext = createContext<Nullable<IFormCommitRegistry>>(null);

/**
 * A registry for a form to own and hand to its fields.
 *
 * Submission is the one moment a form knows a value was meant, and the fields cannot see it. Rather than asking each
 * of them to be told by name - which is a call site that compiles perfectly while doing nothing - the fields say they
 * are interested and the form tells all of them at once.
 *
 * @returns The registry, stable for the life of the form.
 */
export function useFormCommitRegistry(): IFormCommitRegistry {
  const members = useRef<Set<() => void>>(new Set());

  const join = useCallback((commit: () => void): (() => void) => {
    members.current.add(commit);

    return () => {
      members.current.delete(commit);
    };
  }, []);

  const commit = useCallback((): void => {
    for (const member of members.current) {
      member();
    }
  }, []);

  return useMemo(() => ({ commit, join }), [commit, join]);
}

/**
 * Asks to be told when the enclosing form is submitted.
 *
 * Does nothing outside a form. The callback is read at the moment it is needed rather than captured, so a caller may
 * pass a fresh one every render without rejoining.
 *
 * @param onCommit - Called on submission.
 */
export function useCommitOnSubmit(onCommit: () => void): void {
  const registry: Nullable<IFormCommitRegistry> = useContext(FormCommitContext);
  const onCommitRef = useRef<() => void>(onCommit);

  onCommitRef.current = onCommit;

  useEffect(() => {
    if (!registry) {
      return;
    }

    return registry.join(() => onCommitRef.current());
  }, [registry]);
}
