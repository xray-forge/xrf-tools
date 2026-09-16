import { describe, expect, it } from "@jest/globals";

import { ConfigsDocument, ConfigsProjectDescriptor } from "@/core/ipc/types/xrf-app";
import {
  ELtxFindingKind,
  LtxAnchoredFinding,
  LtxResolvedIndex,
  LtxResolvedSection,
  LtxSectionSchemeReport,
} from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsDocumentService, EConfigsDocumentMode } from "@/core/ltx/services/document";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { ConfigsSchemeService } from "@/core/ltx/services/scheme";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

function mockDocumentOf(path: string): ConfigsDocument {
  return {
    findings: [],
    structure: { entryPoints: [], includes: [], parseError: null, path, rootEntries: [], sections: [] },
    text: { isNormalized: true, lines: [], path },
  };
}

function mockOpenedService(): IInjectedServiceMockDescriptor<ConfigsDocumentService> {
  const { service, container } = mockInjectedService(ConfigsDocumentService, [
    ConfigsProjectService,
    ConfigsFindingsService,
    ConfigsResolvedService,
    ConfigsSchemeService,
  ]);
  const project = container.get(ConfigsProjectService);

  project.project = project.project.asReady({ sessionId: "session-1" } as ConfigsProjectDescriptor);

  return { service, container };
}

describe("ConfigsDocumentService", () => {
  it("keeps a cleared document empty when its read finishes", async () => {
    let resolve: (document: ConfigsDocument) => void = noop;
    const response = new Promise<ConfigsDocument>((settle) => {
      resolve = settle;
    });

    setMockInvokeResponses({ "plugin:configs|read_document": () => response });

    const { service } = mockOpenedService();
    const reading = service.select("system.ltx");

    service.clear();
    resolve(mockDocumentOf("system.ltx"));
    await reading;

    expect(service.selected).toBeNull();
    expect(service.document.isIdle).toBe(true);
    expect(service.document.value).toBeNull();
  });

  it("keeps a cleared document idle when its read fails", async () => {
    let reject: (error: Error) => void = noop;
    const response = new Promise<ConfigsDocument>((_resolve, fail) => {
      reject = fail;
    });

    setMockInvokeResponses({ "plugin:configs|read_document": () => response });

    const { service } = mockOpenedService();
    const reading = service.select("system.ltx");

    service.clear();
    reject(new Error("Project closed"));
    await reading;

    expect(service.document.isIdle).toBe(true);
    expect(service.document.error).toBeNull();
  });

  it("keeps the current document visible until its replacement arrives", async () => {
    const previous = mockDocumentOf("previous.ltx");
    const replacement = mockDocumentOf("replacement.ltx");
    let resolve: (document: ConfigsDocument) => void = noop;
    const response = new Promise<ConfigsDocument>((settle) => {
      resolve = settle;
    });

    setMockInvokeResponses({ "plugin:configs|read_document": previous });

    const { service } = mockOpenedService();

    await service.select("previous.ltx");
    setMockInvokeResponses({ "plugin:configs|read_document": () => response });

    const reading = service.select("replacement.ltx");

    expect(service.document.isLoading).toBe(true);
    expect(service.document.value).toEqual(previous);

    resolve(replacement);
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

      setMockInvokeResponses({
        "plugin:configs|read_document": mockDocumentOf(entry),
        "plugin:configs|list_resolved_sections": index,
        "plugin:configs|list_findings": findings,
        "plugin:configs|read_section_scheme": report,
      });

      const { service, container } = mockOpenedService();
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
      let finish: () => void = noop;
      const pending = new Promise<void>((resolve, reject) => {
        finish = () => (outcome === "succeed" ? resolve() : reject(new Error("Late read failed")));
      });

      setMockInvokeResponses({
        "plugin:configs|read_document": () => pending.then(() => mockDocumentOf("other.ltx")),
        "plugin:configs|read_resolved_sections": () => pending.then(() => [section]),
        "plugin:configs|list_findings": () => pending.then(() => findings),
        "plugin:configs|read_section_scheme": () => pending.then(() => report),
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

      finish();
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
    let finish: (index: LtxResolvedIndex) => void = noop;
    const pending = new Promise<LtxResolvedIndex>((resolve) => {
      finish = resolve;
    });

    setMockInvokeResponses({ "plugin:configs|list_resolved_sections": () => pending });

    const { service, container } = mockOpenedService();
    const resolved = container.get(ConfigsResolvedService);
    const reading = resolved.open("system.ltx");

    service.clear();
    finish({ dialect: "ltx", diagnostics: [], entry: "system.ltx", sections: [] });
    await reading;

    expect(resolved.entry).toBeNull();
    expect(resolved.index.isIdle).toBe(true);
    expect(resolved.index.value).toBeNull();
  });
});
