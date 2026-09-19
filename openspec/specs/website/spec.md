# website Specification

## Purpose

Keep the public website usable at its canonical custom domain.

## Requirements

### Requirement: Serve the website from the custom-domain root

Ink SHALL build website links and assets for the root of its canonical custom domain.

#### Scenario: Custom domain loads styled pages {#WEB-001}

- GIVEN the website is deployed to `https://ink.doriankarter.com`
- WHEN a visitor opens the home page or documentation
- THEN canonical metadata identifies the custom domain
- AND navigation and generated assets resolve from the domain root without an `/ink` path prefix

### Requirement: Describe the available product

Ink SHALL describe its commands and capabilities as available behavior without prototype, roadmap, or development-status language.

#### Scenario: Documentation reflects current behavior {#WEB-002}

- GIVEN the public home page and documentation
- WHEN a visitor reads the product overview, installation guide, or reference pages
- THEN implemented behavior is described directly in the present tense
- AND the copy does not characterize Ink as planned, proposed, experimental, or under development

### Requirement: Communicate focused Vim positioning

The home page SHALL present Ink as a fast, focused way to use familiar Vim editing in script prompts without claiming to be a full editor.

#### Scenario: Home page explains Ink's purpose {#WEB-003}

- GIVEN a visitor who is comfortable with Vim
- WHEN the visitor reads the home page
- THEN the primary message connects script input with familiar Vim muscle memory
- AND the supporting copy mentions visual selection and undo and redo
- AND the page distinguishes Ink from a full editor
