use std::fs;

fn release_workflow() -> String {
    fs::read_to_string(".github/workflows/release-please.yml")
        .expect("read release workflow from repository root")
}

fn ci_workflow() -> String {
    fs::read_to_string(".github/workflows/ci.yml").expect("read CI workflow from repository root")
}

#[test]
fn rel_001_release_pr_runs_ci() {
    let workflow = release_workflow();
    let ci = ci_workflow();

    assert!(ci.contains("pull_request:"));
    assert!(
        workflow.contains("permission-contents: write\n          permission-pull-requests: write")
    );
    assert!(workflow.contains(
        "uses: googleapis/release-please-action@45996ed1f6d02564a971a2fa1b5860e934307cf7 # v5.0.0\n        with:\n          # App-authored release PRs trigger the pull_request CI workflow.\n          token: ${{ steps.release_app_token.outputs.token }}"
    ));
}

#[test]
fn rel_002_successful_release_merge_publishes_all_artifacts() {
    let workflow = release_workflow();

    assert!(workflow.contains("github.event.workflow_run.conclusion == 'success'"));
    assert!(workflow.contains("github.event.workflow_run.event == 'push'"));
    assert!(workflow.contains("release_sha: ${{ steps.release-context.outputs.release_sha }}"));
    assert!(workflow.contains("tag_verified: ${{ steps.verify-tag.outputs.verified }}"));
    assert_eq!(
        workflow
            .matches("RELEASE_SHA: ${{ steps.release-context.outputs.release_sha }}")
            .count(),
        1
    );
    assert_eq!(
        workflow
            .matches("RELEASE_SHA: ${{ needs.release-please.outputs.release_sha }}")
            .count(),
        2
    );
    assert_eq!(
        workflow
            .matches("ref: ${{ needs.release-please.outputs.release_sha }}")
            .count(),
        2
    );
    assert_eq!(
        workflow
            .matches("ref: ${{ github.event.workflow_run.head_sha }}")
            .count(),
        1
    );
    for target in [
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
    ] {
        assert!(workflow.contains(target), "missing release target {target}");
    }
    assert!(workflow.contains("install_args: rust mr-boxington"));
    assert!(workflow.contains("gh release upload \"${TAG_NAME}\" dist/ink_* --clobber"));
    assert!(!workflow.contains("gh release upload \"${TAG_NAME}\" dist/* --clobber"));
    assert!(workflow.contains("- build-release"));
    assert!(workflow.contains("gh release edit \"${TAG_NAME}\" --draft=false"));
}

#[test]
fn rel_003_unfinished_draft_resumes_automatically() {
    let workflow = release_workflow();

    assert!(workflow.contains("- name: Resolve new or unfinished release"));
    assert!(workflow.contains("GH_TOKEN: ${{ steps.release_app_token.outputs.token }}"));
    assert!(workflow.contains("jq -r '.[\".\"]' .release-please-manifest.json"));
    assert!(workflow.contains("gh release view \"${tag_name}\" --json isDraft,targetCommitish"));
    assert!(
        workflow.contains("if [[ \"$(jq -r '.isDraft' <<< \"${release_json}\")\" != \"true\" ]]")
    );
    assert!(workflow.contains("release_created: ${{ steps.release-context.outputs.active }}"));
}
