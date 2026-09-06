import {
  apiErrorFixture,
  archivedEntryViewFixture,
  auditFindingFixture,
  clientCapabilitiesFixture,
  contractVersionsFixture,
  entryMutationClearFixture,
  entryMutationOmittedFixture,
  entryViewFixture,
  generatorParamsFixture,
  revisionViewFixture,
  unsupportedFutureVersionsFixture,
  vaultSummaryFixture,
} from './generated/vault-api-contract-fixtures';
import type {
  ApiErrorDto,
  AuditFindingDto,
  ClientCapabilitiesDto,
  ContractVersions,
  EntryDto,
  GeneratorConfigDto,
  RevisionDto,
  UpdateEntryDto,
  VaultInfo,
} from './ipc';

type ExtraKeys<Actual, Expected> = Exclude<keyof Actual, keyof Expected>;
type ReadonlyValue<T> = T extends Array<infer Item> ? readonly Item[] : T;
type ReadonlyContract<T> = {
  readonly [Key in keyof T]: ReadonlyValue<T[Key]>;
};
type ExactContract<Actual, Expected> = Actual extends ReadonlyContract<Expected>
  ? ExtraKeys<Actual, Expected> extends never
    ? Actual
    : never
  : never;

function contractFixture<Expected>() {
  return <Actual extends ReadonlyContract<Expected>>(value: ExactContract<Actual, Expected>) =>
    value;
}

export const checkedDesktopIpcContractFixtures = {
  apiError: contractFixture<ApiErrorDto>()(apiErrorFixture),
  archivedEntryView: contractFixture<EntryDto>()(archivedEntryViewFixture),
  auditFinding: contractFixture<AuditFindingDto>()(auditFindingFixture),
  clientCapabilities: contractFixture<ClientCapabilitiesDto>()(clientCapabilitiesFixture),
  contractVersions: contractFixture<ContractVersions>()(contractVersionsFixture),
  entryMutationClear: contractFixture<UpdateEntryDto>()(entryMutationClearFixture),
  entryMutationOmitted: contractFixture<UpdateEntryDto>()(entryMutationOmittedFixture),
  entryView: contractFixture<EntryDto>()(entryViewFixture),
  generatorParams: contractFixture<GeneratorConfigDto>()(generatorParamsFixture),
  revisionView: contractFixture<RevisionDto>()(revisionViewFixture),
  unsupportedFutureVersions: contractFixture<ContractVersions>()(unsupportedFutureVersionsFixture),
  vaultSummary: contractFixture<VaultInfo>()(vaultSummaryFixture),
} as const;
