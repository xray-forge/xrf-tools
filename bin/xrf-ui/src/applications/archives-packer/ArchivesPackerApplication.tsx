import { default as TuneIcon } from "@mui/icons-material/Tune";
import { Alert, Box, CircularProgress, Divider, Stack, Typography } from "@mui/material";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo, useState } from "react";

import {
  PACKER_SECTIONS_PANEL_LABEL,
  PackerSectionsMenu,
} from "@/applications/archives-packer/components/PackerSectionsMenu";
import { PackerToolbarActions } from "@/applications/archives-packer/components/PackerToolbarActions";
import { ArchivesPackResult } from "@/applications/archives-packer/components/packing/ArchivesPackResult";
import { PackerConfirmSummary } from "@/applications/archives-packer/components/packing/PackerConfirmSummary";
import { PackerHeaderSection } from "@/applications/archives-packer/components/sections/PackerHeaderSection";
import { PackerOptionsSection } from "@/applications/archives-packer/components/sections/PackerOptionsSection";
import { PackerOutputSection } from "@/applications/archives-packer/components/sections/PackerOutputSection";
import { PackerSelectionSection } from "@/applications/archives-packer/components/sections/PackerSelectionSection";
import { EPackerSection, PackerService } from "@/applications/archives-packer/services/packer";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveOutputPath, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/EditorBusyContext";
import { useEditorDirty } from "@/core/shell/EditorDirtyContext";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { useEditorPanels } from "@/core/shell/panel/context";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Nullable } from "@/lib/types/general";

/** Filter used by both configuration dialogs, so import and export agree on what a config is. */
const CONFIG_FILTERS = [{ name: "Packing configuration", extensions: ["ltx"] }];

export function ArchivesPackerApplication(): ReactElement {
  const pathsService: PathsService = useInjection(PathsService);
  const packerService: PackerService = useInjection(PackerService);

  const [isConfirming, setIsConfirming] = useState<boolean>(false);

  const config: Nullable<ArchivePackConfig> = packerService.config;
  const isBusy: boolean = packerService.isBusy;

  const job: Nullable<IJobState> = packerService.job;

  const source: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_PACKER,
    id: "source",
    title: "Select directory to pack",
    isDirectory: true,
    isDisabled: isBusy,
    seed: () => resolvePathRole(EPathRole.GAMEDATA, pathsService.paths),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_PACKER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isBusy,
    seed: () => resolveOutputPath(EApplicationId.ARCHIVES_PACKER, pathsService.paths),
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
      filters: CONFIG_FILTERS,
    })) as Nullable<string>;

    if (selected) {
      await packerService.importConfig(selected);
    }
  }, [packerService]);

  const onExport = useCallback(async () => {
    const selected: Nullable<string> = await save({ title: "Export packing configuration", filters: CONFIG_FILTERS });

    if (selected) {
      await packerService.exportConfig(selected);
    }
  }, [packerService]);

  const onPack = useCallback(async () => {
    if (!resolved) {
      return;
    }

    setIsConfirming(false);

    await packerService.pack(resolved);
  }, [packerService, resolved]);

  // Drawn by the shell beside every other application's navigation, rather than as a column of this
  // application's own. The menu reads the open section from the service, so this registers once.
  useEditorPanels(
    () => [
      {
        icon: <TuneIcon />,
        id: "packer-sections",
        isOpenByDefault: true,
        label: PACKER_SECTIONS_PANEL_LABEL,
        render: () => <PackerSectionsMenu />,
        side: "left",
      },
    ],
    []
  );

  useEditorStatus([
    packerService.configName ?? "no configuration",
    ...(packerService.isDirty ? ["unsaved changes"] : []),
    ...(packerService.result ? [`${packerService.result.volumes.length} volume(s)`] : []),
  ]);

  useEditorBusy(isBusy);

  useEditorDirty(packerService.isDirty ? 1 : 0);

  if (!config) {
    return (
      <EditorLayout toolbar={<EditorToolbar />}>
        <Box sx={{ display: "flex", flexGrow: 1, alignItems: "center", justifyContent: "center" }}>
          <CircularProgress size={28} />
        </Box>
      </EditorLayout>
    );
  }

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={packerService.configName ?? "New configuration"}
          actions={
            <PackerToolbarActions
              isBusy={isBusy}
              isPackDisabled={isBusy || !resolved || Boolean(packerService.volumeSizeError)}
              onImport={() => void onImport()}
              onExport={() => void onExport()}
              onPack={() => setIsConfirming(true)}
            />
          }
        />
      }
    >
      <Box sx={{ flexGrow: 1, minWidth: 0, overflowY: "auto", p: 3 }}>
        <Stack spacing={2} sx={{ maxWidth: 860 }}>
          {packerService.error ? <Alert severity={"error"}>{packerService.error}</Alert> : null}

          {job ? <JobProgressView job={job} onCancel={packerService.cancel} /> : null}

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

          {packerService.result ? (
            <>
              <Divider />
              <Typography variant={"subtitle2"}>Last run</Typography>
              <ArchivesPackResult result={packerService.result} />
            </>
          ) : null}
        </Stack>
      </Box>

      {resolved ? (
        <ConfirmDialog
          isOpen={isConfirming}
          isDestructive={true}
          maxWidth={"sm"}
          title={"Pack archives?"}
          description={<PackerConfirmSummary config={resolved} />}
          confirmLabel={"Pack"}
          onConfirm={() => void onPack()}
          onClose={() => setIsConfirming(false)}
        />
      ) : null}
    </EditorLayout>
  );
}
