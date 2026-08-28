import { open } from "@tauri-apps/plugin-dialog";
import { exists } from "@tauri-apps/plugin-fs";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { assetsCommands } from "@/core/bindings/commands/assets";
import { XrayRootProbe } from "@/core/bindings/types/xrf-vfs";
import { SettingsPathField } from "@/core/settings/components/SettingsPathField";
import { resolvePathRole } from "@/core/settings/lib/path";
import { describeRootProbe } from "@/core/settings/lib/root-probe";
import { IWorkspacePathDescriptor } from "@/core/settings/lib/workspace-path";
import { PathsService } from "@/core/settings/services/paths";
import { Nullable } from "@/lib/types/general";

/**
 * What to say about a path that is set.
 *
 * A failed check is reported as nothing rather than as a problem, the way every other path check in the application
 * treats one: not being able to ask is not the same as the answer being no.
 */
async function describe(path: IWorkspacePathDescriptor, value: string): Promise<Nullable<string>> {
  try {
    if (path.isRoot) {
      const probe: XrayRootProbe = await assetsCommands.probeRoot(value);

      return describeRootProbe(probe);
    }

    return (await exists(value)) ? null : "Path does not exist";
  } catch {
    return null;
  }
}

export interface ISettingsPathRowProps {
  path: IWorkspacePathDescriptor;
}

/**
 * One configured path, with what it currently amounts to.
 */
export function SettingsPathRow({ path }: ISettingsPathRowProps): ReactElement {
  const pathsService: PathsService = useInjection(PathsService);

  const value: Nullable<string> = pathsService.getPath(path.id);
  const paths = pathsService.paths;

  const [derived, setDerived] = useState<Nullable<string>>(null);
  const [fact, setFact] = useState<Nullable<string>>(null);

  const onSelect = useCallback(async () => {
    const selected: Nullable<string> = await open({ title: path.title, directory: true });

    if (selected) {
      pathsService.setPath(path.id, selected);
    }
  }, [path.id, path.title, pathsService]);

  const onClear = useCallback(() => pathsService.setPath(path.id, null), [path.id, pathsService]);

  useEffect(() => {
    if (value || !path.derivedFrom) {
      setDerived(null);

      return;
    }

    let isCurrent: boolean = true;

    resolvePathRole(path.derivedFrom, paths)
      .then((resolved: Nullable<string>) => isCurrent && setDerived(resolved))
      .catch(() => isCurrent && setDerived(null));

    return () => {
      isCurrent = false;
    };
  }, [path.derivedFrom, paths, value]);

  useEffect(() => {
    if (!value) {
      setFact(null);

      return;
    }

    let isCurrent: boolean = true;

    void describe(path, value).then((described: Nullable<string>) => isCurrent && setFact(described));

    return () => {
      isCurrent = false;
    };
  }, [path, value]);

  return (
    <SettingsPathField
      label={path.label}
      description={path.description}
      value={value}
      placeholder={derived}
      fact={fact}
      onSelect={() => void onSelect()}
      onClear={onClear}
    />
  );
}
