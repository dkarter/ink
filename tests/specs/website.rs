use std::fs;

#[test]
fn web_001_custom_domain_loads_styled_pages() {
    let config = fs::read_to_string("website/astro.config.mjs").expect("read Astro config");
    let home = fs::read_to_string("website/src/pages/index.astro").expect("read home page");
    let social = fs::read_to_string("website/src/social.mjs").expect("read social metadata");

    assert!(social.contains("site = \"https://ink.doriankarter.com\""));
    assert!(config.contains("process.env.INK_SITE_BASE || \"/\""));
    assert!(config.contains("`${site}${publicBase}${socialImagePath}`"));
    assert!(home.contains("`${site}${socialImagePath}`"));
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
        "website/src/og-image.svg",
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

#[test]
fn web_004_shared_links_render_a_large_image_card() {
    let home = fs::read_to_string("website/src/pages/index.astro").expect("read home page");
    let config = fs::read_to_string("website/astro.config.mjs").expect("read Astro config");
    let social = fs::read_to_string("website/src/social.mjs").expect("read social metadata");
    let image = fs::read("website/public/og-image.png").expect("read social image");

    for expected in [
        "socialImagePath = \"/og-image.png\"",
        "socialImageType = \"image/png\"",
        "socialImageWidth = \"1200\"",
        "socialImageHeight = \"630\"",
        "socialImageAlt = \"Ink brings Vim muscle memory to input prompts in shell scripts\"",
    ] {
        assert!(
            social.contains(expected),
            "incorrect social value: {expected}"
        );
    }

    for expected in [
        "<meta property=\"og:image\" content={socialImage} />",
        "<meta property=\"og:image:type\" content={socialImageType} />",
        "<meta property=\"og:image:width\" content={socialImageWidth} />",
        "<meta property=\"og:image:height\" content={socialImageHeight} />",
        "<meta property=\"og:image:alt\" content={socialImageAlt} />",
        "<meta name=\"twitter:image\" content={socialImage} />",
        "<meta name=\"twitter:image:alt\" content={socialImageAlt} />",
    ] {
        assert!(
            home.contains(expected),
            "incorrect home metadata: {expected}"
        );
    }

    for expected in [
        r#"property: "og:image", content: socialImage"#,
        r#"property: "og:image:type", content: socialImageType"#,
        r#"property: "og:image:width", content: socialImageWidth"#,
        r#"property: "og:image:height", content: socialImageHeight"#,
        r#"property: "og:image:alt", content: socialImageAlt"#,
        r#"name: "twitter:image", content: socialImage"#,
        r#"name: "twitter:image:alt", content: socialImageAlt"#,
    ] {
        assert!(
            config.contains(expected),
            "incorrect docs metadata: {expected}"
        );
    }

    assert_eq!(&image[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(u32::from_be_bytes(image[16..20].try_into().unwrap()), 1200);
    assert_eq!(u32::from_be_bytes(image[20..24].try_into().unwrap()), 630);
}
