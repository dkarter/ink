# releases Specification

## Purpose

Publish tested Ink binaries from release PRs without a separate manual release path.

## Requirements

### Requirement: Validate generated release changes

Ink SHALL create release PRs with an identity whose events can trigger the repository CI workflow.

#### Scenario: Release PR runs CI {#REL-001}

- GIVEN Release Please has prepared version and changelog changes
- WHEN it opens or updates the release PR
- THEN the release PR triggers the same CI checks as any other pull request

### Requirement: Publish approved release artifacts

Ink SHALL process releases only after main CI succeeds, bind every artifact to the commit selected by Release Please, and publish only after every supported binary artifact is uploaded.

#### Scenario: Successful release merge publishes all artifacts {#REL-002}

- GIVEN a release PR is merged and a current main commit passes CI
- WHEN Release Please creates its tag and draft GitHub Release
- THEN Ink verifies the tag against the release commit selected by Release Please
- AND Ink uploads Linux x86_64, Linux arm64, macOS arm64, and Windows x86_64 archives built from that release commit
- AND Ink publishes the draft only after every artifact build succeeds

#### Scenario: Unfinished draft resumes automatically {#REL-003}

- GIVEN Release Please already created the manifest version's tag and draft GitHub Release
- AND a previous artifact or publication step did not complete
- WHEN a current main commit passes CI
- THEN Ink resumes that draft through the same verified build and publication path
