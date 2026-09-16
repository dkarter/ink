import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";

const site = "https://dkarter.github.io";
const base = process.env.INK_SITE_BASE || "/ink";

export default defineConfig({
  site,
  base,
  prefetch: false,
  integrations: [
    starlight({
      title: "ink",
      description: "A planned Vim-style prompt editor for the terminal.",
      favicon: "/favicon.svg",
      head: [
        { tag: "meta", attrs: { property: "og:image", content: `${site}${base}/og-image.svg` } },
        { tag: "meta", attrs: { name: "twitter:card", content: "summary_large_image" } },
        { tag: "meta", attrs: { name: "twitter:image", content: `${site}${base}/og-image.svg` } },
      ],
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/dkarter/ink" }],
      customCss: ["./src/styles/starlight.css"],
      components: { SiteTitle: "./src/components/SiteTitle.astro" },
      editLink: { baseUrl: "https://github.com/dkarter/ink/edit/main/website/" },
      lastUpdated: true,
      disable404Route: true,
      sidebar: [
        {
          label: "Start",
          items: [
            { label: "Overview", slug: "docs" },
            { label: "Install", slug: "docs/install" },
            { label: "Quick start", slug: "docs/quick-start" },
          ],
        },
        {
          label: "Guide",
          items: [
            { label: "CLI", slug: "docs/cli" },
            { label: "Vim modes", slug: "docs/vim-modes" },
            { label: "Configuration", slug: "docs/configuration" },
            { label: "Themes", slug: "docs/themes" },
          ],
        },
      ],
    }),
  ],
});
