import { DialogFilter } from "@tauri-apps/plugin-dialog";
import { exists } from "@tauri-apps/plugin-fs";
import { useCallback, useEffect, useState } from "react";

import { EApplicationId } from "@/core/routing/application";
import { usePathState } from "@/core/ui/form/file-picker/use-path-state";
import { IPathSeed, TPathSeed, usePathSeed } from "@/core/ui/form/use-path-seed";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Nullable } from "@/lib/types/general";

const STORAGE_PREFIX: string = "xrf.form.";
const VALIDATE_DEBOUNCE_MS: number = 250;

export interface IPathFieldOptions {
  /** The application that owns this field. */
  application: EApplicationId;
  /** Names the field inside its own form, for example `source`. Only unique within the application. */
  id: string;
  title?: string;
  filters?: Nullable<Array<DialogFilter>>;
  isDirectory?: boolean;
  isSave?: boolean;
  isDisabled?: boolean;
  isRequired?: boolean;
  /** Produces a first guess when nothing has been remembered yet, usually from the configured paths. */
  seed?: TPathSeed;
}

export interface IPathField {
  value: Nullable<string>;
  error: Nullable<string>;
  isValid: boolean;
  select: () => Promise<void>;
  clear: () => void;
  setValue: (value: Nullable<string>) => void;
}

/**
 * Manages a remembered, validated filesystem path for one form field.
 *
 * What is stored is only ever what a caller asked to store: the remembered path is read once, during the first render,
 * and written only by `select`, `clear`, and `setValue`. Nothing about mounting, remounting, or guessing touches
 * storage, so no session can overwrite or erase what an earlier one remembered while its own state is still empty.
 *
 * Clearing asks for the guess again rather than leaving the field blank, which is what makes clearing read as "back to
 * the default" once configured paths supply those defaults.
 *
 * @param options - Field identity, dialog behavior, and validation options.
 * @param options.application - Application the field belongs to, used to scope persistence.
 * @param options.id - Field name within that application, used for persistence.
 * @param options.title - Path dialog title.
 * @param options.filters - File filters shown by the dialog.
 * @param options.isDirectory - Whether the dialog selects a directory.
 * @param options.isSave - Whether the path may identify a new output file.
 * @param options.isDisabled - Whether selection is disabled.
 * @param options.isRequired - Whether an empty path is invalid.
 * @param options.seed - Async fallback used when no path is stored.
 * @returns The current path state, validation result, and field actions.
 */
export function usePathField({
  application,
  id,
  title,
  filters = null,
  isDirectory = false,
  isSave = false,
  isDisabled = false,
  isRequired = true,
  seed,
}: IPathFieldOptions): IPathField {
  const storageKey: string = `${STORAGE_PREFIX}${application}.${id}`;

  const [value, setPath, selectPath] = usePathState({
    title,
    filters,
    isDirectory,
    isSave,
    isDisabled,
    initial: () => getLocalStorageValue(storageKey),
  });
  const [error, setError] = useState<Nullable<string>>(null);

  // A guess fills the field without being stored, so every session re-derives it until the user picks a path.
  const { request: requestSeed, supersede: supersedeSeed }: IPathSeed = usePathSeed({ seed, onSeeded: setPath });

  const setValue = useCallback(
    (next: Nullable<string>): void => {
      supersedeSeed();
      setPath(next);
      setLocalStorageValue(storageKey, next);
    },
    [setPath, storageKey, supersedeSeed]
  );

  const clear = useCallback((): void => {
    setPath(null);
    setLocalStorageValue(storageKey, null);
    requestSeed();
  }, [requestSeed, setPath, storageKey]);

  const select = useCallback(async (): Promise<void> => {
    // Superseded as the dialog opens rather than after it answers: the pick reaches the underlying state before this
    // call resumes, so a guess arriving in between would land on top of it.
    supersedeSeed();

    const picked: Nullable<string> = await selectPath();

    // Cancelling leaves both the state and what was remembered as they were, which for an untouched field means the
    // guess it would otherwise have shown.
    if (picked === null) {
      if (value === null) {
        requestSeed();
      }

      return;
    }

    setLocalStorageValue(storageKey, picked);
  }, [requestSeed, selectPath, storageKey, supersedeSeed, value]);

  // Asked for against storage rather than against the field, because the rule is that nothing was ever remembered
  // here - which is also what the field holds at this point, and stays true if the key is ever replaced.
  useEffect(() => {
    if (getLocalStorageValue(storageKey) === null) {
      requestSeed();
    }
  }, [requestSeed, storageKey]);

  useEffect(() => {
    if (!value || isSave) {
      setError(null);

      return;
    }

    let isCurrent: boolean = true;

    const handle = setTimeout(() => {
      exists(value)
        .then((isPresent) => isCurrent && setError(isPresent ? null : "Path does not exist"))
        // A failed check is not proof of absence, so it is reported as unknown rather than missing.
        .catch(() => isCurrent && setError(null));
    }, VALIDATE_DEBOUNCE_MS);

    return () => {
      isCurrent = false;
      clearTimeout(handle);
    };
  }, [value, isSave]);

  return {
    value,
    error,
    isValid: (!isRequired || Boolean(value)) && !error,
    select,
    clear,
    setValue,
  };
}
