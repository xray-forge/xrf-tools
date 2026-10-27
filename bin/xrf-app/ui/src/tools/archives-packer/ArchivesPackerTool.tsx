import { DialogFilter, open, save } from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ArchivePackConfig } from "@xrf/ipc/types/xrf-pack";
import { Nullable } from "@xrf/types";
import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { TuneIcon } from "@xrf/ui/icons/tune";
import { Alert } from "@xrf/ui/material/alert";
import { Divider } from "@xrf/ui/material/divider";
import { Stack } from "@xrf/ui/material/stack";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import { ARCHIVE_CONFIG_EXTENSIONS, withArchiveConfigExtension } from "@/core/archive/lib";
import { ARCHIVE_CONFIG_EXPORT_FILTERS } from "@/core/archive/lib/config-format";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EToolId } from "@/core/routing/tool";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";
import { ToolLoader } from "@/core/shell/chrome/ToolLoader";
import { useShellPanels, useShellStatus } from "@/core/shell/contributions";
import { useToolLifecycle } from "@/core/shell/lifecycle";
import { IPathField, usePathField } from "@/core/ui/form";
import { ArchivesPackerService, EPackerSection } from "@/tools/archives-packer/services/archives-packer.service";

import { PackerSectionsMenu } from "./components/PackerSectionsMenu";
import { PackerToolbarActions } from "./components/PackerToolbarActions";
import { ArchivesPackResult, PackerConfirmSummary } from "./components/packing";
import {
  PackerHeaderSection,
  PackerOptionsSection,
  PackerOutputSection,
  PackerSelectionSection,
} from "./components/sections";

/** Filter the open dialog offers: one entry listing every format, so browsing shows all configurations at once. */
const IMPORT_CONFIG_FILTERS: Array<DialogFilter> = [
  { name: "Packing configuration", extensions: [...ARCHIVE_CONFIG_EXTENSIONS] },
];

export function ArchivesPackerTool(): ReactElement {
  const packerService: ArchivesPackerService = useInjection(ArchivesPackerService);

  const [isConfirming, setIsConfirming] = useState<boolean>(false);

  /** Whether the user agreed to replace the volumes the destination already holds. Asked again per confirmation. */
  const [isForced, setIsForced] = useState<boolean>(false);

  const config: Nullable<ArchivePackConfig> = packerService.config;
  const job: Nullable<IJobState> = packerService.operation.job;

  const isBusy: boolean = packerService.isBusy;
  const isPacking: boolean = Boolean(job);

  const source: IPathField = usePathField({
    tool: EToolId.ARCHIVES_PACKER,
    id: "source",
    title: "Select directory to pack",
    isDirectory: true,
    isDisabled: isBusy,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.ARCHIVES_PACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isBusy,
    seed: () => resolveOutputPath(EToolId.ARCHIVES_PACKER),
  });

  /** The configuration as it would be packed, with the fields the editor owns folded back in. */
  const resolved: Nullable<ArchivePackConfig> = useMemo(() => {
    if (!config || !source.value || !destination.value) {
      return null;
    }

    return {
      ...config,
      source: source.value,
      destination: destination.value,
      maxVolumeSize: packerService.volumeSizeBytes,
    };
  }, [config, source.value, destination.value, packerService.volumeSizeBytes]);

  const onImport = useCallback(async () => {
    const selected: Nullable<string> = (await open({
      title: "Import packing configuration",
      filters: IMPORT_CONFIG_FILTERS,
    })) as Nullable<string>;

    if (selected) {
      await packerService.importConfig(selected);
    }
  }, [packerService]);

  const onExport = useCallback(async () => {
    const selected: Nullable<string> = await save({
      title: "Export packing configuration",
      filters: ARCHIVE_CONFIG_EXPORT_FILTERS,
    });

    if (selected) {
      await packerService.exportConfig(withArchiveConfigExtension(selected));
    }
  }, [packerService]);

  /** Opens the confirmation and asks what the destination already holds, so the summary can say so. */
  const onConfirmPack = useCallback(() => {
    if (!resolved) {
      return;
    }

    setIsForced(false);
    setIsConfirming(true);

    void packerService.checkDestination(resolved);
  }, [packerService, resolved]);

  const onPack = useCallback(async () => {
    if (!resolved) {
      return;
    }

    setIsConfirming(false);

    source.commit();
    destination.commit();

    await packerService.pack(resolved, isForced);
  }, [destination, packerService, resolved, isForced, source]);

  useEffect(() => {
    packerService.resetResult();
  }, [source.value, destination.value, packerService]);

  // Drawn by the shell beside every other tool's navigation, rather than as a column of this
  // tool's own. The menu reads the open section from the service, so this registers once.
  useShellPanels(
    () => [
      {
        icon: <TuneIcon />,
        id: "packer-sections",
        isOpenByDefault: true,
        label: "Configuration",
        render: () => <PackerSectionsMenu />,
        side: "left",
      },
    ],
    []
  );

  useShellStatus([
    packerService.configName ?? "no configuration",
    ...(packerService.isDirty ? ["unsaved changes"] : []),
    ...(packerService.operation.result ? [`${packerService.operation.result.volumes.length} volume(s)`] : []),
  ]);

  useToolLifecycle({
    isBusy: isBusy || isPacking,
    dirtyCount: packerService.isDirty ? 1 : 0,
    save: null,
  });

  if (!config) {
    return <ToolLoader />;
  }

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={packerService.configName ?? "New configuration"}
          actions={
            <PackerToolbarActions
              isBusy={isBusy || isPacking}
              isPackDisabled={isBusy || isPacking || !resolved || Boolean(packerService.volumeSizeError)}
              onImport={() => void onImport()}
              onExport={() => void onExport()}
              onPack={onConfirmPack}
            />
          }
        />
      }
    >
      <div className={"min-w-0 grow overflow-y-auto p-6"}>
        <Stack className={"max-w-reading"} spacing={2}>
          {packerService.error ? <Alert severity={"error"}>{packerService.error}</Alert> : null}

          {job ? <JobProgressView job={job} onCancel={packerService.operation.cancel} /> : null}

          {packerService.section === EPackerSection.OUTPUT ? (
            <PackerOutputSection
              config={config}
              source={source}
              destination={destination}
              isDisabled={isBusy}
              onChange={packerService.patchConfig}
            />
          ) : null}

          {packerService.section === EPackerSection.SELECTION ? (
            <PackerSelectionSection config={config} isDisabled={isBusy} onChange={packerService.patchConfig} />
          ) : null}

          {packerService.section === EPackerSection.HEADER ? (
            <PackerHeaderSection config={config} isDisabled={isBusy} onChange={packerService.patchConfig} />
          ) : null}

          {packerService.section === EPackerSection.OPTIONS ? (
            <PackerOptionsSection
              config={config}
              maxVolumeSizeMegabytes={packerService.maxVolumeSizeMegabytes}
              volumeSize={packerService.volumeSize}
              volumeSizeError={packerService.volumeSizeError}
              isDisabled={isBusy}
              onVolumeSizeChange={packerService.setVolumeSize}
              onChange={packerService.patchConfig}
            />
          ) : null}

          {packerService.operation.result ? (
            <>
              <Divider />
              <Typography variant={"subtitle2"}>Last run</Typography>
              <ArchivesPackResult result={packerService.operation.result} />
            </>
          ) : null}
        </Stack>
      </div>

      {resolved ? (
        <ConfirmDialog
          isOpen={isConfirming}
          isDestructive={true}
          isConfirmDisabled={packerService.publishedVolumes.length > 0 && !isForced}
          maxWidth={"sm"}
          title={"Pack archives?"}
          description={
            <PackerConfirmSummary
              config={resolved}
              publishedVolumes={packerService.publishedVolumes}
              isForced={isForced}
              onForceChange={setIsForced}
            />
          }
          confirmLabel={"Pack"}
          onConfirm={() => void onPack()}
          onClose={() => setIsConfirming(false)}
        />
      ) : null}
    </EditorLayout>
  );
}
