import { Nullable } from "@xrf/types";
import { useCallback } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { EToolId, getToolRoute } from "@/core/routing/tool";
import { useRequestLeave } from "@/core/shell/lifecycle";
import { handOverPathFields } from "@/core/ui/form/hand-over-path-fields";

/** Opens a tool holding the paths handed over, by its fields' ids; a field handed null opens empty. */
export type THandOverPaths = (tool: EToolId, fields: Readonly<Record<string, Nullable<string>>>) => void;

/**
 * Opens another tool with its path fields filled, for a step from one tool's result into the next tool of a workflow,
 * such as from a scene to its build. It leaves the open tool as its own back does, so work running holds it and unsaved
 * edits are asked about; only once it leaves are the paths remembered under the other tool's fields and that tool
 * navigated to, so a refused leave keeps what the other tool remembered.
 *
 * @returns Opens a tool with paths.
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
