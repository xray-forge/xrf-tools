import { useCallback, useRef, useState } from "react";

import { Nullable } from "@/lib/types/general";

/** Reports the first character a language cannot hold, or `null` when the value is writable. */
export type TValidateText = (language: string, text: string) => Promise<Nullable<string>>;

export interface IUseTranslationValidationOptions {
  /** Logical file whose cells are on screen. */
  file: Nullable<string>;
  /** Language being translated into, whose code page decides what a value may hold. */
  language: string;
  /** Performs the validation, usually the service call that reaches the backend. */
  validateText: TValidateText;
}

export interface ITranslationValidation {
  /** What is wrong with a cell as it currently stands, or `null`. */
  getErrorOf: (id: string) => Nullable<string>;
  /** Validate a value just committed to a cell, superseding whatever that cell had in flight. */
  validate: (id: string, value: string) => void;
}

/** One cell's identity as a single key. `\0` occurs in no logical path, language tag, or translation id. */
function toCellKey(file: string, language: string, id: string): string {
  return `${file}\0${language}\0${id}`;
}

/**
 * Validation errors for the cells on screen, which arrive too late to be believed on their own.
 *
 * @param options - The cells in view and how to validate one.
 * @param options.file - Logical file whose cells are on screen.
 * @param options.language - Language being translated into.
 * @param options.validateText - Performs the validation.
 * @returns Reading a cell's error, and validating a committed value.
 */
export function useTranslationValidation({
  file,
  language,
  validateText,
}: IUseTranslationValidationOptions): ITranslationValidation {
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Held rather than closed over, because callers pass a fresh arrow every render and validating must not
  // re-identify the commit handler with it.
  const validateTextRef = useRef<TValidateText>(validateText);

  /** Commit number of the newest edit to each cell, stamped onto the request that edit started. */
  const commitsRef = useRef<Map<string, number>>(new Map());

  validateTextRef.current = validateText;

  const getErrorOf = useCallback(
    (id: string): Nullable<string> => (file ? (errors[toCellKey(file, language, id)] ?? null) : null),
    [errors, file, language]
  );

  const validate = useCallback(
    (id: string, value: string): void => {
      if (!file) {
        return;
      }

      const key: string = toCellKey(file, language, id);
      const commit: number = (commitsRef.current.get(key) ?? 0) + 1;

      commitsRef.current.set(key, commit);

      validateTextRef
        .current(language, value)
        .then((error: Nullable<string>) => {
          if (commitsRef.current.get(key) !== commit) {
            return;
          }

          setErrors((current: Record<string, string>) => {
            // Most answers say nothing is wrong, and re-identifying the map for one of those would redraw
            // the whole table on every commit.
            if ((current[key] ?? null) === error) {
              return current;
            }

            const { [key]: _cleared, ...rest } = current;

            return error ? { ...rest, [key]: error } : rest;
          });
        })
        .catch(() => undefined);
    },
    [file, language]
  );

  return { getErrorOf, validate };
}
