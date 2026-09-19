import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import {
  site,
  socialImageAlt,
  socialImageHeight,
  socialImagePath,
  socialImageType,
  socialImageWidth,
} from "./src/social.mjs";

const base = process.env.INK_SITE_BASE || "/";
const publicBase = base === "/" ? "" : base.replace(/\/$/, "");
const socialImage = `${site}${publicBase}${socialImagePath}`;

export default defineConfig({
  site,
  base,
  prefetch: false,
  integrations: [
    starlight({
      title: "ink",
      description: "A Vim-style prompt editor for composable terminal workflows.",
      favicon: "/favicon.svg",
      head: [
        {
          tag: "meta",
          attrs: { property: "og:image", content: socialImage },
        },
        { tag: "meta", attrs: { property: "og:image:type", content: socialImageType } },
        { tag: "meta", attrs: { property: "og:image:width", content: socialImageWidth } },
        { tag: "meta", attrs: { property: "og:image:height", content: socialImageHeight } },
        { tag: "meta", attrs: { property: "og:image:alt", content: socialImageAlt } },
        { tag: "meta", attrs: { name: "twitter:card", content: "summary_large_image" } },
        {
          tag: "meta",
          attrs: { name: "twitter:image", content: socialImage },
        },
        { tag: "meta", attrs: { name: "twitter:image:alt", content: socialImageAlt } },
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
