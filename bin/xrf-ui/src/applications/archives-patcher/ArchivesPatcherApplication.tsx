import { TextField } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback, useEffect, useState } from "react";

import { ArchivesPatchResult } from "@/applications/archives-patcher/components/ArchivesPatchResult";
import { PatcherService } from "@/applications/archives-patcher/services/patcher";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { ArchivesPatchRequest } from "@/core/bindings/types/xrf-app";
import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import {
  CheckboxFormRow,
  FormRow,
  IPathField,
  PathFormRow,
  StringListFormRow,
  usePathField,
  useRememberedValue,
} from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Whether the form's submit compares or publishes, remembered as the text the store keeps. */
type TPatcherMode = "preview" | "publish";

const PATCHER_MODES: ReadonlyArray<TPatcherMode> = ["preview", "publish"];

export function ArchivesPatcherApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const patcherService: PatcherService = useInjection(PatcherService);

  const job: Nullable<IJobState> = patcherService.operation.job;
  const isRunning: boolean = patcherService.operation.isRunning;

  // Off by default: the installation supplies both sides, which is the case a path pair cannot express at all.
  const [isDeliveringOwnTree, setIsDeliveringOwnTree] = useState<boolean>(false);

  const [defaults, setDefaults] = useState<Nullable<ArchivePatchConfig>>(null);
  const [name, setName] = useState<string>("patch");
  const [include, setInclude] = useState<Array<string>>([]);
  const [ignore, setIgnore] = useState<Array<string>>([]);

  // Remembered as text because that is what the store holds; the form only ever asks whether it is "preview".
  const [mode, setMode] = useRememberedValue<TPatcherMode>({
    application: EApplicationId.ARCHIVES_PATCHER,
    id: "mode",
    fallback: "preview",
    allowed: PATCHER_MODES,
  });

  const isPreviewOnly: boolean = mode === "preview";

  const input: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_PATCHER,
    id: "input",
    title: "Select the game installation",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const target: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_PATCHER,
    id: "target",
    title: "Select the tree the patch delivers",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_PATCHER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
    seed: () => resolveOutputPath(EApplicationId.ARCHIVES_PATCHER),
  });

  const toRequest = useCallback((): Nullable<ArchivesPatchRequest> => {
    // Everything the format owns - the volume ceiling, the mode, the extension, the mountable header - comes from the
    // backend's own defaults, so this form never becomes a second definition of them.
    if (input.value === null || destination.value === null || defaults === null) {
      return null;
    }

    return {
      config: {
        ...defaults,
        input: input.value,
        // Absent is the useful default: the installation's own loose gamedata is what the modder has been editing.
        target: isDeliveringOwnTree ? target.value : null,
        destination: destination.value,
        name,
        include,
        ignore,
      },
      isForced: false,
      isStrict: false,
      isVerifyingPayload: false,
    };
  }, [defaults, destination.value, ignore, include, input.value, isDeliveringOwnTree, name, target.value]);

  const onSubmit = useCallback(async () => {
    const request: Nullable<ArchivesPatchRequest> = toRequest();

    if (!request) {
      return;
    }

    log.info("Comparing archives, preview only:", isPreviewOnly);

    input.commit();
    destination.commit();

    if (isDeliveringOwnTree) {
      target.commit();
    }

    await (isPreviewOnly ? patcherService.compare(request) : patcherService.patch(request));
  }, [destination, input, isDeliveringOwnTree, isPreviewOnly, log, patcherService, target, toRequest]);

  // What the result panel offers after a preview: the same form, committed, without retyping any of it.
  const onWrite = useCallback(async () => {
    const request: Nullable<ArchivesPatchRequest> = toRequest();

    if (request) {
      await patcherService.patch(request);
    }
  }, [patcherService, toRequest]);

  const onCancel = useCallback(() => patcherService.operation.cancel(), [patcherService]);

  // Read once: these are facts about the format, so nothing that happens in the form can change them.
  useEffect(() => {
    void archivesCommands.defaultPatchConfig().then((config: ArchivePatchConfig) => {
      setDefaults(config);
      setName(config.name);
    });
  }, []);

  // Changing any input invalidates whatever the previous run reported.
  useEffect(() => {
    patcherService.operation.reset();
  }, [destination.value, ignore, include, input.value, isDeliveringOwnTree, name, patcherService, target.value]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={
        defaults === null ||
        !input.isValid ||
        !destination.isValid ||
        !name.trim() ||
        (isDeliveringOwnTree && !target.isValid)
      }
      title={"Build an archive patch"}
      description={
        "Packs what your loose gamedata changes about an installation into archive volumes that override it when the " +
        "engine mounts them."
      }
      error={patcherService.operation.error ?? undefined}
      submitLabel={isPreviewOnly ? "Compare" : "Build patch"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        patcherService.operation.result ? (
          <ArchivesPatchResult
            result={patcherService.operation.result}
            outputPath={destination.value}
            isDisabled={isRunning}
            onWrite={onWrite}
          />
        ) : null
      }
      onSubmit={onSubmit}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Game"}
        description={"The installation to patch; its loose gamedata is compared against its own archives"}
        field={input}
      />

      <CheckboxFormRow
        label={"Deliver another tree"}
        description={"Build the patch from a separate gamedata folder instead of the installation's loose files"}
        isChecked={isDeliveringOwnTree}
        isDisabled={isRunning}
        onChange={setIsDeliveringOwnTree}
      />

      {isDeliveringOwnTree ? (
        <PathFormRow
          isDisabled={isRunning}
          label={"Deliver"}
          description={"The gamedata tree the patch should carry"}
          field={target}
        />
      ) : null}

      <PathFormRow
        isDisabled={isRunning}
        label={"Output"}
        description={"Directory the patch volumes are written into, outside the game"}
        field={destination}
      />

      <FormRow label={"Volume name"} description={"Volumes are written as <name>.db0, <name>.db1 and so on"}>
        <TextField
          size={"small"}
          fullWidth
          value={name}
          disabled={isRunning}
          onChange={(event: ChangeEvent<HTMLInputElement>) => setName(event.target.value)}
        />
      </FormRow>

      <CheckboxFormRow
        label={"Preview only"}
        description={"Report what differs without writing any volume"}
        isChecked={isPreviewOnly}
        isDisabled={isRunning}
        onChange={(isChecked: boolean) => setMode(isChecked ? "preview" : "publish")}
      />

      <StringListFormRow
        label={"Only compare"}
        description={"Logical prefixes the comparison is restricted to, such as configs"}
        values={include}
        addLabel={"Add prefix"}
        emptyLabel={"The whole of both worlds is compared."}
        placeholder={"configs"}
        isDisabled={isRunning}
        onChange={setInclude}
      />

      <StringListFormRow
        label={"Ignore"}
        description={"Logical prefixes dropped from the comparison"}
        values={ignore}
        addLabel={"Add prefix"}
        emptyLabel={"Nothing is ignored."}
        placeholder={"configs\\text"}
        isDisabled={isRunning}
        onChange={setIgnore}
      />
    </PickerForm>
  );
}
