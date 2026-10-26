import { useCallback } from "react";
import { NavigateFunction, useLocation, useNavigate } from "react-router-dom";

export type TChunkTabChange = (event: unknown, value: string) => void;

/**
 * Resolves the active chunk sub-table from the route.
 *
 * @param basePath - Route prefix shared by the chunk tabs.
 * @param tabs - Valid route segments for the chunk.
 * @param fallback - Tab used when the route has no valid tab segment.
 * @returns The active tab and a route-backed tab-change handler.
 */
export function useChunkTab(basePath: string, tabs: Array<string>, fallback: string): [string, TChunkTabChange] {
  const navigate: NavigateFunction = useNavigate();

  const { pathname } = useLocation();

  const segment: string = pathname.split("/").filter(Boolean).pop() ?? "";
  const activeTab: string = tabs.includes(segment) ? segment : fallback;

  const onChangeTab: TChunkTabChange = useCallback(
    (_: unknown, value: string) => navigate(`${basePath}/${value}`, { replace: true }),
    [basePath, navigate]
  );

  return [activeTab, onChangeTab];
}
