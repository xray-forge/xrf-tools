import { default as TuneIcon } from "@mui/icons-material/Tune";
import { Alert, Box, Divider, Stack, Typography } from "@mui/material";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import { ArchivesPatchResult } from "@/applications/archives-patcher/components/ArchivesPatchResult";
import {
  PATCHER_SECTIONS_PANEL_LABEL,
  PatcherSectionsMenu,
} from "@/applications/archives-patcher/components/PatcherSectionsMenu";
import { PatcherToolbarActions } from "@/applications/archives-patcher/components/PatcherToolbarActions";
import { PatcherConfirmSummary } from "@/applications/archives-patcher/components/patching/PatcherConfirmSummary";
import { PatcherComparisonSection } from "@/applications/archives-patcher/components/sections/PatcherComparisonSection";
import { PatcherHeaderSection } from "@/applications/archives-patcher/components/sections/PatcherHeaderSection";
import { PatcherOptionsSection } from "@/applications/archives-patcher/components/sections/PatcherOptionsSection";
import { PatcherOutputSection } from "@/applications/archives-patcher/components/sections/PatcherOutputSection";
import { PatcherSelectionSection } from "@/applications/archives-patcher/components/sections/PatcherSelectionSection";
import { PATCH_CONFIG_EXTENSIONS, withPatchConfigExtension } from "@/applications/archives-patcher/lib/patch-config";
import { EPatcherSection, PatcherService } from "@/applications/archives-patcher/services/patcher";
import { ArchivesPatchRequest } from "@/core/bindings/types/xrf-app";
import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorLifecycle } from "@/core/shell/editor-lifecycle";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";
import { IPathField, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Filter the open dialog offers: one entry listing every format, so browsing shows all configurations at once. */
const IMPORT_CONFIG_FILTERS = [{ name: "Patching configuration", extensions: [...PATCH_CONFIG_EXTENSIONS] }];

/** Filters the save dialog offers, one entry per format. */
const EXPORT_CONFIG_FILTERS = PATCH_CONFIG_EXTENSIONS.map((it) => ({ name: it, extensions: [it] }));

export function ArchivesPatcherApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const patcherService: PatcherService = useInjection(PatcherService);

  const config: Nullable<ArchivePatchConfig> = patcherService.config;
  const job: Nullable<IJobState> = patcherService.operation.job;

  const isRunning: boolean = patcherService.operation.isRunning;
  const isBusy: boolean = isRunning || patcherService.isBusy;

  const [isVerifyingPayload, setIsVerifyingPayload] = useState<boolean>(false);
  /** Which run the confirmation is for, or null while it is closed. */
  const [confirming, setConfirming] = useState<Nullable<"compare" | "patch">>(null);
  const [isForced, setIsForced] = useState<boolean>(false);

  const input: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_PATCHER,
    id: "input",
    title: "Select the game installation",
    isDirectory: true,
    isDisabled: isBusy,
  });

  const target: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_PATCHER,
    id: "target",
    title: "Select the tree the patch delivers",
    isDirectory: true,
    isDisabled: isBusy,
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_PATCHER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isBusy,
    seed: () => resolveOutputPath(EApplicationId.ARCHIVES_PATCHER),
  });

  /** Whether the patch is built from a folder of its own, which is what a non-null target means. */
  const isDeliveringOwnTree: boolean = Boolean(config && config.target !== null);

  const request: Nullable<ArchivesPatchRequest> = useMemo(() => {
    if (!config || input.value === null || destination.value === null) {
      return null;
    }

    if (isDeliveringOwnTree && !target.value) {
      return null;
    }

    return {
      config: {
        ...config,
        input: input.value,
        target: isDeliveringOwnTree ? target.value : null,
        destination: destination.value,
      },
      isForced,
      isVerifyingPayload,
    };
  }, [config, destination.value, input.value, isDeliveringOwnTree, isForced, isVerifyingPayload, target.value]);

  const onImport = useCallback(async () => {
    const selected: Nullable<string> = await open({
      title: "Import patching configuration",
      multiple: false,
      filters: IMPORT_CONFIG_FILTERS,
    });

    if (typeof selected === "string") {
      await patcherService.importConfig(selected);
    }
  }, [patcherService]);

  const onExport = useCallback(async () => {
    const selected: Nullable<string> = await save({
      title: "Export patching configuration",
      filters: EXPORT_CONFIG_FILTERS,
    });

    if (selected) {
      await patcherService.exportConfig(withPatchConfigExtension(selected));
    }
  }, [patcherService]);

  /** Opens the confirmation, asking what the output already holds so the summary can say so. */
  const onConfirm = useCallback(
    (kind: "compare" | "patch") => {
      if (!request) {
        return;
      }

      setIsForced(false);
      setConfirming(kind);

      if (kind === "patch") {
        void patcherService.checkDestination(request.config);
      }
    },
    [patcherService, request]
  );

  const onRun = useCallback(async () => {
    if (!request || !confirming) {
      return;
    }

    log.info("Running patcher:", confirming);

    setConfirming(null);

    input.commit();
    destination.commit();

    if (isDeliveringOwnTree) {
      target.commit();
    }

    await (confirming === "compare" ? patcherService.compare(request) : patcherService.patch(request));
  }, [confirming, destination, input, isDeliveringOwnTree, log, patcherService, request, target]);

  const onCancel = useCallback(() => patcherService.operation.cancel(), [patcherService]);

  const onVolumeSizeChange = useCallback(
    (value: string) => {
      patcherService.setVolumeSize(value);
    },
    [patcherService]
  );

  // Changing what is compared invalidates whatever the previous run reported.
  useEffect(() => {
    patcherService.operation.reset();
  }, [destination.value, input.value, patcherService, target.value]);

  // Drawn by the shell beside every other application's navigation rather than as a column of this application's
  // own. The menu reads the open section from the service, so this registers once.
  useEditorPanels(
    () => [
      {
        icon: <TuneIcon />,
        id: "patcher-sections",
        isOpenByDefault: true,
        label: PATCHER_SECTIONS_PANEL_LABEL,
        render: () => <PatcherSectionsMenu />,
        side: "left",
      },
    ],
    []
  );

  useEditorStatus([
    patcherService.configName ?? "no configuration",
    ...(patcherService.isDirty ? ["unsaved changes"] : []),
    ...(patcherService.operation.result
      ? [`${patcherService.operation.result.added.length + patcherService.operation.result.modified.length} carried`]
      : []),
  ]);

  useEditorLifecycle({
    isBusy,
    dirtyCount: patcherService.isDirty ? 1 : 0,
    save: null,
  });

  if (!config) {
    return <ApplicationLoader />;
  }

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={patcherService.configName ?? "New configuration"}
          actions={
            <PatcherToolbarActions
              isBusy={isBusy}
              isRunDisabled={isBusy || !request || !config.name.trim() || Boolean(patcherService.volumeSizeError)}
              onImport={() => void onImport()}
              onExport={() => void onExport()}
              onCompare={() => onConfirm("compare")}
              onPatch={() => onConfirm("patch")}
            />
          }
        />
      }
    >
      <Box sx={{ flexGrow: 1, minWidth: 0, overflowY: "auto", p: 3 }}>
        <Stack spacing={2} sx={{ maxWidth: 860 }}>
          {patcherService.error ? <Alert severity={"error"}>{patcherService.error}</Alert> : null}

          {patcherService.operation.error ? <Alert severity={"error"}>{patcherService.operation.error}</Alert> : null}

          {job ? <JobProgressView job={job} onCancel={onCancel} /> : null}

          {patcherService.section === EPatcherSection.COMPARISON ? (
            <PatcherComparisonSection
              config={config}
              input={input}
              target={target}
              isDisabled={isBusy}
              onChange={patcherService.patchConfig}
            />
          ) : null}

          {patcherService.section === EPatcherSection.OUTPUT ? (
            <PatcherOutputSection
              config={config}
              destination={destination}
              isDisabled={isBusy}
              onChange={patcherService.patchConfig}
            />
          ) : null}

          {patcherService.section === EPatcherSection.SELECTION ? (
            <PatcherSelectionSection config={config} isDisabled={isBusy} onChange={patcherService.patchConfig} />
          ) : null}

          {patcherService.section === EPatcherSection.HEADER ? (
            <PatcherHeaderSection config={config} isDisabled={isBusy} onChange={patcherService.patchConfig} />
          ) : null}

          {patcherService.section === EPatcherSection.OPTIONS ? (
            <PatcherOptionsSection
              config={config}
              maxVolumeSizeMegabytes={patcherService.maxVolumeSizeMegabytes}
              volumeSize={patcherService.volumeSize}
              volumeSizeError={patcherService.volumeSizeError}
              isVerifyingPayload={isVerifyingPayload}
              isForced={isForced}
              isDisabled={isBusy}
              onVolumeSizeChange={onVolumeSizeChange}
              onVerifyingPayloadChange={setIsVerifyingPayload}
              onForcedChange={setIsForced}
              onChange={patcherService.patchConfig}
            />
          ) : null}

          {patcherService.operation.result ? (
            <>
              <Divider />
              <Typography variant={"subtitle2"}>Last run</Typography>
              <ArchivesPatchResult result={patcherService.operation.result} outputPath={destination.value} />
            </>
          ) : null}
        </Stack>
      </Box>

      {request ? (
        <ConfirmDialog
          isOpen={confirming !== null}
          isDestructive={confirming === "patch"}
          isConfirmDisabled={confirming === "patch" && patcherService.publishedVolumes.length > 0 && !isForced}
          maxWidth={"sm"}
          title={confirming === "compare" ? "Compare archives?" : "Write patch?"}
          description={
            <PatcherConfirmSummary
              config={request.config}
              isPreviewOnly={confirming === "compare"}
              publishedVolumes={patcherService.publishedVolumes}
              isForced={isForced}
              onForceChange={setIsForced}
            />
          }
          confirmLabel={confirming === "compare" ? "Compare" : "Patch"}
          onConfirm={() => void onRun()}
          onClose={() => setConfirming(null)}
        />
      ) : null}
    </EditorLayout>
  );
}
