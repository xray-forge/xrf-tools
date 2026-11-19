import { Nullable } from "@xrf/types";
import { useCallback } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { EToolId, getToolRoute } from "@/core/routing/tool";
import { useRequestLeave } from "@/core/shell/lifecycle";
import { handOverPathFields } from "@/core/ui/form/hand-over-path-fields";

/** Opens a tool holding the paths handed over, by its fields' ids; a field handed null opens empty. */
export type THandOverPaths = (tool: EToolId, fields: Readonly<Record<string, Nullable<string>>>) => void;

/**
 * Opens another tool with its path fields filled, for a workflow step such as from a scene to its build. It leaves the
 * open tool as its own back does, so running work holds it and unsaved edits are asked about. The paths are stored
 * only once it leaves, so a refused leave keeps what the other tool remembered.
 */
export function useHandOverPaths(): THandOverPaths {
  const navigate: NavigateFunction = useNavigate();
  const requestLeave: (leave: () => void) => void = useRequestLeave();

  return useCallback(
    (tool: EToolId, fields: Readonly<Record<string, Nullable<string>>>): void => {
      requestLeave(() => {
        handOverPathFields(tool, fields);
        navigate(getToolRoute(tool), { replace: true });
      });
    },
    [navigate, requestLeave]
  );
}
