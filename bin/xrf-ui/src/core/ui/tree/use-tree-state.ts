import { useCallback, useState } from "react";

import { getAncestorDirectoryIds } from "@/core/ui/tree/path-tree";
import { Nullable } from "@/lib/types/general";

export interface IUseTreeState {
  expandedIds: ReadonlySet<string>;
  /** The row the tree draws as chosen, which is also where the keyboard stands. */
  selectedId: Nullable<string>;
  toggleExpanded: (id: string) => void;
  expand: (id: string) => void;
  expandAll: (ids: Iterable<string>) => void;
  select: (id: Nullable<string>) => void;
  /** Selects a node and opens whatever stands above it. */
  reveal: (id: string) => void;
}

export interface IUseTreeStateOptions {
  /** Ids open before anything is touched, for a tree that starts partly unfolded. */
  initialExpandedIds?: Iterable<string>;
}

/**
 * Expansion and selection for one tree.
 *
 * Held here rather than in `VirtualizedTree` so a consumer can select and reveal a row it opened elsewhere, and
 * held once rather than in four menus that were each spelling the same toggle. Selection is inert by contract:
 * moving it never opens anything, which is what makes it safe to move on every arrow key.
 *
 * @param options - Starting expansion.
 * @param options.initialExpandedIds - Ids open before anything is touched.
 * @returns The state the tree renders from, and the operations that move it.
 */
export function useTreeState({ initialExpandedIds }: IUseTreeStateOptions = {}): IUseTreeState {
  const [expandedIds, setExpandedIds] = useState<ReadonlySet<string>>(() => new Set(initialExpandedIds));
  const [selectedId, setSelectedId] = useState<Nullable<string>>(null);

  const toggleExpanded = useCallback((id: string): void => {
    setExpandedIds((current: ReadonlySet<string>) => {
      const next: Set<string> = new Set(current);

      if (!next.delete(id)) {
        next.add(id);
      }

      return next;
    });
  }, []);

  const expandAll = useCallback((ids: Iterable<string>): void => {
    setExpandedIds((current: ReadonlySet<string>) => {
      const missing: Array<string> = [...ids].filter((id: string) => !current.has(id));

      // Additive, and identical when nothing is missing, so collapsing by hand afterwards stays collapsed.
      return missing.length ? new Set([...current, ...missing]) : current;
    });
  }, []);

  const expand = useCallback((id: string): void => expandAll([id]), [expandAll]);

  const select = useCallback((id: Nullable<string>): void => setSelectedId(id), []);

  const reveal = useCallback(
    (id: string): void => {
      expandAll(getAncestorDirectoryIds(id));
      setSelectedId(id);
    },
    [expandAll]
  );

  return { expandedIds, selectedId, toggleExpanded, expand, expandAll, select, reveal };
}
