import { VerifierService } from "@/applications/configs-verifier/services/verifier";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ConfigsVerifierApplication as Component } from "./ConfigsVerifierApplication";

export const container: ContainerDefinition = { bindings: [VerifierService] };
