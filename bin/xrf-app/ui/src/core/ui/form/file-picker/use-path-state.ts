import { isTauri } from "@tauri-apps/api/core";
import { Nullable } from "@xrf/types";
import { useCallback, useRef, useState } from "react";

import { IFileFilter } from "@/core/path/file-filters";
import { askToOpen, askToOpenDirectory, askToSave } from "@/core/path/native-dialog";

export interface IPathStateOptions {
  title?: string;
  filters?: Nullable<Array<IFileFilter>>;
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
}

export type TPathState = [Nullable<string>, (value: Nullable<string>) => void, () => Promise<Nullable<string>>];

/**
 * Holds a picked path and the action that fills it, opening the dialog where the held path leads. No dialog opens
 * while disabled, and a cancelled one leaves the current value alone.
 *
 * @returns The selected path, its setter, and the selection action reporting what was picked.
 */
export function usePathState({
  title = "Provide path",
  filters = null,
  isDisabled = false,
  isDirectory = false,
  isSave = false,
  initial,
}: IPathStateOptions = {}): TPathState {
  const [pathState, setPathState] = useState<Nullable<string>>(() => initial?.() ?? null);

  // Held rather than closed over, so the callback stays stable while still opening where the path points right now.
  const pathRef = useRef<Nullable<string>>(pathState);

  pathRef.current = pathState;

  // Filters are declared inline by callers, so their identity changes every render. Comparing by
  // content keeps the callback stable without asking every caller to memoise.
  const filtersKey: string = JSON.stringify(filters);

  const onSelectPath = useCallback(async (): Promise<Nullable<string>> => {
    if (isDisabled || !isTauri()) {
      return null;
    }

    const start: Nullable<string> = pathRef.current;

    // `isDirectory` decides which dialog, because a save dialog cannot pick one and asking for a
    // directory is not negotiable — an output directory is still a directory when it is also new.
    // `isSave` only relaxes the existence check, which `usePathField` applies. Branching on `isSave`
    // first put a file-name dialog in front of every screen whose destination is a folder.
    const pathResponse: Nullable<string> = isDirectory
      ? await askToOpenDirectory({ title, start, isDestination: isSave })
      : isSave
        ? await askToSave({ title, start, filters })
        : await askToOpen({ title, start, filters });

    // Cancelling resolves null. Keeping the previous value is deliberate: replacing a good path with
    // nothing because someone opened the dialog and thought better of it is never what was wanted.
    if (pathResponse) {
      setPathState(pathResponse);
    }

    // Reported rather than only stored, so a caller can persist exactly what the user picked.
    return pathResponse;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [title, isDirectory, isDisabled, isSave, filtersKey]);

  return [pathState, setPathState, onSelectPath];
}
