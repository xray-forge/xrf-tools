import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { useCallback, useEffect } from "react";
import { NavigateFunction, useLocation, useNavigate } from "react-router-dom";

import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";
import { ShellToolService } from "@/core/shell/services/shell-tool";

/** Keeps the shell's open tool on the route shown, and leaving it on the router, while the caller is mounted. */
export function useShellToolRoute(): void {
  const shellToolService: ShellToolService = useInjection(ShellToolService);
  const tool: Nullable<IToolDescriptor> = useCurrentTool();
  const navigate: NavigateFunction = useNavigate();

  const { pathname } = useLocation();

  // The launcher's route exactly: a mistyped url opens no tool either, and is not the launcher.
  const isLauncher: boolean = !tool && pathname === "/";

  const onLeave = useCallback(() => navigate("/", { replace: true }), [navigate]);

  useEffect(() => shellToolService.follow(tool, isLauncher, onLeave), [shellToolService, tool, isLauncher, onLeave]);

  useEffect(() => () => shellToolService.release(onLeave), [shellToolService, onLeave]);
}
