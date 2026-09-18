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
