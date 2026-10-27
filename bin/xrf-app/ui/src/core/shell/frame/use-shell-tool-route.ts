import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { useCallback, useEffect } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";
import { ShellToolService } from "@/core/shell/services/shell-tool";

/** Keeps the shell's open tool on the route shown, and leaving it on the router, while the caller is mounted. */
export function useShellToolRoute(): void {
  const shellToolService: ShellToolService = useInjection(ShellToolService);
  const tool: Nullable<IToolDescriptor> = useCurrentTool();
  const navigate: NavigateFunction = useNavigate();

  const onLeave = useCallback(() => navigate("/", { replace: true }), [navigate]);

  useEffect(() => shellToolService.follow(tool?.id ?? null, onLeave), [shellToolService, tool, onLeave]);

  useEffect(() => () => shellToolService.release(onLeave), [shellToolService, onLeave]);
}
