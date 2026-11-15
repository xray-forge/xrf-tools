import { useCallback } from "react";

/**
 * @param value - The whole value.
 * @param onChange - Told the whole value it becomes.
 * @returns Sets part of the value, the rest as it was.
 */
export function usePartialChange<T extends object>(value: T, onChange: (value: T) => void): (part: Partial<T>) => void {
  return useCallback((part: Partial<T>) => onChange({ ...value, ...part }), [value, onChange]);
}
