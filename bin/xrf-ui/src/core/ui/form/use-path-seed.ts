import { useCallback, useRef } from "react";

import { Nullable, Optional } from "@/lib/types/general";

/** Produces a first guess at a path, usually from the configured paths. */
export type TPathSeed = () => Promise<Nullable<string>>;

export interface IPathSeedOptions {
  /** Produces the guess. Absent where a field has nothing to guess from. */
  seed?: TPathSeed;
  /** Receives a guess that is still the newest one asked for. */
  onSeeded: (path: string) => void;
}

export interface IPathSeed {
  /** Guess now, superseding whatever was already in flight. */
  request: () => void;
  /** Drop whatever is in flight, because something better than a guess has arrived. */
  supersede: () => void;
}

/**
 * A guess at a path that can be superseded before it arrives.
 *
 * Guessing is asynchronous and a user is not obliged to wait for it, so the answer may be worthless by the time it
 * comes back: they may have chosen a path, typed one, or emptied the field meanwhile. Each guess therefore carries the
 * generation it started in and writes only while that generation is still current. A guess that fails writes nothing
 * at all rather than clearing the field, for the same reason.
 *
 * @param options - The guess and where its answer goes.
 * @param options.seed - Produces the guess.
 * @param options.onSeeded - Receives a guess that has not been superseded.
 * @returns Asking for a guess, and dropping one.
 */
export function usePathSeed({ seed, onSeeded }: IPathSeedOptions): IPathSeed {
  // Held rather than closed over, so both actions stay stable while callers pass a fresh arrow every render.
  const seedRef = useRef<Optional<TPathSeed>>(seed);
  const onSeededRef = useRef<(path: string) => void>(onSeeded);

  /** Stamped onto each guess as it starts, and bumped by anything that makes one stale. */
  const generationRef = useRef<number>(0);

  seedRef.current = seed;
  onSeededRef.current = onSeeded;

  const supersede = useCallback((): void => {
    generationRef.current += 1;
  }, []);

  const request = useCallback((): void => {
    const resolveSeed: Optional<TPathSeed> = seedRef.current;

    if (!resolveSeed) {
      return;
    }

    const generation: number = (generationRef.current += 1);

    resolveSeed()
      .then((guessed: Nullable<string>) => {
        if (guessed && generation === generationRef.current) {
          onSeededRef.current(guessed);
        }
      })
      .catch(() => undefined);
  }, []);

  return { request, supersede };
}
