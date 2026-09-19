use std::fs;

#[test]
fn web_001_custom_domain_loads_styled_pages() {
    let config = fs::read_to_string("website/astro.config.mjs").expect("read Astro config");
    let home = fs::read_to_string("website/src/pages/index.astro").expect("read home page");

    assert!(config.contains("const site = \"https://ink.doriankarter.com\";"));
    assert!(config.contains("process.env.INK_SITE_BASE || \"/\""));
    assert!(config.contains("`${site}${publicBase}/og-image.svg`"));
    assert!(home.contains("const site = \"https://ink.doriankarter.com\";"));
    assert!(!home.contains("https://dkarter.github.io/ink/"));
}

#[test]
fn web_002_documentation_reflects_current_behavior() {
    let sources = [
        "website/src/pages/index.astro",
        "website/src/content/docs/index.mdx",
        "website/src/content/docs/install.md",
        "website/src/content/docs/quick-start.md",
        "website/src/content/docs/cli.md",
        "website/src/content/docs/vim-modes.md",
        "website/src/content/docs/configuration.md",
        "website/src/content/docs/themes.md",
        "website/astro.config.mjs",
        "website/public/og-image.svg",
    ]
    .map(|path| fs::read_to_string(path).expect("read website source"))
    .join("\n")
    .to_lowercase();

    for stale in [
        "experimental",
        "prototype",
        "roadmap",
        "planned",
        "proposed",
        "specified",
        "intended",
        "should",
        "early development",
        "work in progress",
        "still being written",
        "being designed",
        "in development",
        "bootstrap project",
        "not implemented",
        "future prompt",
    ] {
        assert!(!sources.contains(stale), "stale product copy: {stale}");
    }
}

#[test]
fn web_003_home_page_explains_inks_purpose() {
    let home = fs::read_to_string("website/src/pages/index.astro")
        .expect("read home page")
        .to_lowercase();

    for message in [
        "script input",
        "vim muscle memory",
        "visual line",
        "visual block",
        "undo and redo",
        "full editor",
    ] {
        assert!(
            home.contains(message),
            "missing home page message: {message}"
        );
    }
}
