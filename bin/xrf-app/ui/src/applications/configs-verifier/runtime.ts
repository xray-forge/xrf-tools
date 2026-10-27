import { ConfigsVerifierService } from "@/applications/configs-verifier/services/configs-verifier.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ConfigsVerifierApplication as Component } from "./ConfigsVerifierApplication";

export const container: ContainerDefinition = { bindings: [ConfigsVerifierService] };
