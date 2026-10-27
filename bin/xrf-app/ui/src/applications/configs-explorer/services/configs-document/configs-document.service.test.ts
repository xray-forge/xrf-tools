import { describe, expect, it } from "@jest/globals";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ConfigsDocument } from "@xrf/ipc/types/xrf-app-configs";
import { ELtxFindingKind } from "@xrf/ipc/types/xrf-ltx";
import {
  LtxAnchoredFinding,
  LtxResolvedIndex,
  LtxResolvedSection,
  LtxSectionSchemeReport,
} from "@xrf/ipc/types/xrf-ltx-inspect";

import { mockRestoredConfigsProject } from "@/applications/configs-explorer/fixtures/configs-project";
import {
  ConfigsDocumentService,
  EConfigsDocumentMode,
} from "@/applications/configs-explorer/services/configs-document";
import { ConfigsFindingsService } from "@/applications/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/applications/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/applications/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/applications/configs-explorer/services/configs-scheme";
import { mockSessionRead } from "@/fixtures/mocks/session.mocks";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

function mockDocumentOf(path: string): ConfigsDocument {
  return {
    findings: [],
    structure: { entryPoints: [], includes: [], parseError: null, path, rootEntries: [], sections: [] },
    text: { isNormalized: true, lines: [], path },
  };
}

async function mockOpenedService(): Promise<IInjectedServiceMockDescriptor<ConfigsDocumentService>> {
  const { service, container } = mockInjectedService(ConfigsDocumentService, [
    ConfigsProjectService,
    ConfigsFindingsService,
    ConfigsResolvedService,
    ConfigsSchemeService,
  ]);

  await mockRestoredConfigsProject(container.get(ConfigsProjectService));

  return { service, container };
}

describe("ConfigsDocumentService", () => {
  it("keeps a cleared document empty when its read finishes", async () => {
    const response = deferred<ConfigsDocument>();

    mockIpcCommands({ [EConfigsCommand.READ_DOCUMENT]: mockSessionRead(() => response.promise) });

    const { service } = await mockOpenedService();
    const reading = service.select("system.ltx");

    service.clear();
    response.resolve(mockDocumentOf("system.ltx"));
    await reading;

    expect(service.selected).toBeNull();
    expect(service.document.isIdle).toBe(true);
    expect(service.document.value).toBeNull();
  });

  it("keeps a cleared document idle when its read fails", async () => {
    const response = deferred<ConfigsDocument>();

    mockIpcCommands({ [EConfigsCommand.READ_DOCUMENT]: mockSessionRead(() => response.promise) });

    const { service } = await mockOpenedService();
    const reading = service.select("system.ltx");

    service.clear();
    response.reject(new Error("Project closed"));
    await reading;

    expect(service.document.isIdle).toBe(true);
    expect(service.document.error).toBeNull();
  });

  it("keeps the current document visible until its replacement arrives", async () => {
    const previous = mockDocumentOf("previous.ltx");
    const replacement = mockDocumentOf("replacement.ltx");
    const response = deferred<ConfigsDocument>();

    mockIpcCommands({ [EConfigsCommand.READ_DOCUMENT]: mockSessionRead(previous) });

    const { service } = await mockOpenedService();

    await service.select("previous.ltx");
    mockIpcCommands({ [EConfigsCommand.READ_DOCUMENT]: mockSessionRead(() => response.promise) });

    const reading = service.select("replacement.ltx");

    expect(service.document.isLoading).toBe(true);
    expect(service.document.value).toEqual(previous);

    response.resolve(replacement);
    await reading;

    expect(service.document.isReady).toBe(true);
    expect(service.document.value).toEqual(replacement);
  });

  it.each(["succeed", "fail"])(
    "clears the selection and its panels together when pending reads %s",
    async (outcome) => {
      const entry = "system.ltx";
      const section: LtxResolvedSection = { entry, fields: [], name: "wpn_base", origin: entry, parents: [] };
      const index: LtxResolvedIndex = {
        dialect: "ltx",
        diagnostics: [],
        entry,
        sections: [{ fieldCount: 0, name: section.name, origin: entry, parents: [] }],
      };
      const findings: Array<LtxAnchoredFinding> = [
        {
          engineBehaviour: null,
          entry,
          field: null,
          file: entry,
          kind: ELtxFindingKind.SCHEME,
          line: 1,
          message: "Unknown scheme",
          section: section.name,
        },
      ];
      const report: LtxSectionSchemeReport = {
        entry,
        fields: [],
        inheritedFrom: null,
        isDeclared: false,
        isStrict: false,
        scheme: "weapon",
        section: section.name,
      };

      mockIpcCommands({
        [EConfigsCommand.READ_DOCUMENT]: mockSessionRead(mockDocumentOf(entry)),
        [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(index),
        [EConfigsCommand.LIST_FINDINGS]: mockSessionRead(findings),
        [EConfigsCommand.READ_SECTION_SCHEME]: mockSessionRead(report),
      });

      const { service, container } = await mockOpenedService();
      const project = container.get(ConfigsProjectService);
      const resolved = container.get(ConfigsResolvedService);
      const problems = container.get(ConfigsFindingsService);
      const scheme = container.get(ConfigsSchemeService);

      await Promise.all([
        service.select(entry),
        resolved.open(entry),
        problems.open(entry),
        scheme.read(entry, section.name),
      ]);
      service.selectSection(section.name);
      service.setMode(EConfigsDocumentMode.RESOLVED);

      const projectValue = project.project.value;
      const pending = deferred<void>();

      mockIpcCommands({
        [EConfigsCommand.READ_DOCUMENT]: mockSessionRead(() => pending.promise.then(() => mockDocumentOf("other.ltx"))),
        [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead(() => pending.promise.then(() => [section])),
        [EConfigsCommand.LIST_FINDINGS]: mockSessionRead(() => pending.promise.then(() => findings)),
        [EConfigsCommand.READ_SECTION_SCHEME]: mockSessionRead(() => pending.promise.then(() => report)),
      });

      const reads = [
        service.select("other.ltx"),
        resolved.request([section.name]),
        problems.open("other.ltx"),
        scheme.read(entry, "other_section"),
      ];

      expect(service.document.value).not.toBeNull();
      expect(problems.findings.value).toEqual(findings);
      expect(scheme.report.value).toEqual(report);

      const callsBeforeClear = mockInvoke.mock.calls.length;

      service.clear();

      expect(mockInvoke.mock.calls).toHaveLength(callsBeforeClear);
      expect(service.selected).toBeNull();
      expect(resolved.index.isIdle).toBe(true);
      expect(problems.findings.isIdle).toBe(true);
      expect(scheme.report.isIdle).toBe(true);

      const revision = resolved.revision;

      if (outcome === "succeed") {
        pending.resolve();
      } else {
        pending.reject(new Error("Late read failed"));
      }

      await Promise.all(reads);

      expect(service.selected).toBeNull();
      expect(service.selectedSection).toBeNull();
      expect(service.document.isIdle).toBe(true);
      expect(service.document.value).toBeNull();
      expect(service.document.error).toBeNull();
      expect(resolved.entry).toBeNull();
      expect(resolved.index.value).toBeNull();
      expect(resolved.sections.size).toBe(0);
      expect(resolved.revision).toBe(revision);
      expect(problems.entry).toBeNull();
      expect(problems.findings.isIdle).toBe(true);
      expect(problems.findings.value).toEqual([]);
      expect(problems.findings.error).toBeNull();
      expect(scheme.entry).toBeNull();
      expect(scheme.section).toBeNull();
      expect(scheme.report.isIdle).toBe(true);
      expect(scheme.report.value).toBeNull();
      expect(scheme.report.error).toBeNull();
      expect(project.project.value).toBe(projectValue);
      expect(service.mode).toBe(EConfigsDocumentMode.RESOLVED);
    }
  );

  it("abandons an index read when the document selection is cleared", async () => {
    const pending = deferred<LtxResolvedIndex>();

    mockIpcCommands({ [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(() => pending.promise) });

    const { service, container } = await mockOpenedService();
    const resolved = container.get(ConfigsResolvedService);
    const reading = resolved.open("system.ltx");

    service.clear();
    pending.resolve({ dialect: "ltx", diagnostics: [], entry: "system.ltx", sections: [] });
    await reading;

    expect(resolved.entry).toBeNull();
    expect(resolved.index.isIdle).toBe(true);
    expect(resolved.index.value).toBeNull();
  });
});
