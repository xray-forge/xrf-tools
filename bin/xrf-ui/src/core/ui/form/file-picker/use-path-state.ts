import { isTauri } from "@tauri-apps/api/core";
import { DialogFilter, open, save } from "@tauri-apps/plugin-dialog";
import { useCallback, useRef, useState } from "react";

import { Nullable, Optional } from "@/lib/types/general";

export interface IPathStateOptions {
  title?: string;
  filters?: Nullable<Array<DialogFilter>>;
  isDisabled?: boolean;
  isDirectory?: boolean;
  /**
   * Ask where to write instead of what to read, so a destination that does not exist yet is allowed.
   *
   * Says *whether the path must already exist*, never what kind of thing it is — `isDirectory` owns
   * that, and wins. An output directory is both: it names a directory, and it need not be there yet.
   */
  isSave?: boolean;
  /** Produces the starting path, read once on the first render. */
  initial?: () => Nullable<string>;
  /** Where the dialog should open, asked each time it is about to. */
  defaultPath?: () => Promise<Optional<string>>;
}

export type TPathState = [Nullable<string>, (value: Nullable<string>) => void, () => Promise<Nullable<string>>];

/**
 * Holds a picked path and the action that fills it.
 *
 * Barely more than `useState` plus a configured dialog call, and that is the point: the two guards it
 * owns - refusing to open while disabled, and leaving the current value alone when the user cancels.
 *
 * @param options - Dialog and interaction options.
 * @param options.title - Dialog title.
 * @param options.filters - File filters shown by the dialog.
 * @param options.isDisabled - Whether selection is disabled.
 * @param options.isDirectory - Whether the dialog selects a directory.
 * @param options.isSave - Whether the dialog selects an output path.
 * @param options.initial - Starting path, evaluated once.
 * @param options.defaultPath - Where the dialog should open, asked each time.
 * @returns The selected path, its setter, and the selection action reporting what was picked.
 */
export function usePathState({
  title = "Provide path",
  filters = null,
  isDisabled = false,
  isDirectory = false,
  isSave = false,
  initial,
  defaultPath,
}: IPathStateOptions = {}): TPathState {
  const [pathState, setPathState] = useState<Nullable<string>>(() => initial?.() ?? null);

  // Held rather than closed over, so the callback stays stable while callers pass a fresh arrow every render - and
  // still reads the newest one, which is what makes it see the path the field is holding right now.
  const defaultPathRef = useRef<Optional<() => Promise<Optional<string>>>>(defaultPath);

  defaultPathRef.current = defaultPath;

  // Filters are declared inline by callers, so their identity changes every render. Comparing by
  // content keeps the callback stable without asking every caller to memoise.
  const filtersKey: string = JSON.stringify(filters);

  const onSelectPath = useCallback(async (): Promise<Nullable<string>> => {
    if (isDisabled || !isTauri()) {
      return null;
    }

    // Asked as the dialog opens rather than held, because it describes where the field currently points and that
    // changes under this callback. A failure to answer is no answer, never a refusal to open the dialog.
    const from: Optional<string> = await defaultPathRef.current?.().catch(() => undefined);

    // `isDirectory` decides which dialog, because a save dialog cannot pick one and asking for a
    // directory is not negotiable — an output directory is still a directory when it is also new.
    // `isSave` only relaxes the existence check, which `usePathField` applies. Branching on `isSave`
    // first put a file-name dialog in front of every screen whose destination is a folder.
    const pathResponse: Nullable<string> =
      isSave && !isDirectory
        ? await save({ defaultPath: from, title, filters: filters ? filters : undefined })
        : await open({ defaultPath: from, title, filters: filters ? filters : undefined, directory: isDirectory });

    // Cancelling resolves null. Keeping the previous value is deliberate: replacing a good path with
    // nothing because someone opened the dialog and thought better of it is never what was wanted.
    if (pathResponse) {
      setPathState(pathResponse);
    }

    // Reported rather than only stored, so a caller can persist exactly what the user picked.
    return pathResponse ?? null;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [title, isDirectory, isDisabled, isSave, filtersKey]);

  return [pathState, setPathState, onSelectPath];
}
