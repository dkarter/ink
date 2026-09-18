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
