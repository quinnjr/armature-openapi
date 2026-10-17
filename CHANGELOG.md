# Changelog — `armature-openapi`

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Earlier changes are recorded in the workspace [`CHANGELOG.md`](../CHANGELOG.md).

## [Unreleased]

## [0.5.0] - 2026-10-17

### Security

- Swagger UI assets are now pinned to `swagger-ui-dist` 5.33.1 (was 5.10.0)
  and loaded with Subresource Integrity (`integrity="sha384-..."`,
  `crossorigin="anonymous"`), so a tampered or substituted CDN file is refused
  by the browser. The pinned version is exposed as `SWAGGER_UI_VERSION`.
- The Swagger UI page title is now HTML-escaped.
- Replaced the deprecated `serde_yaml` (and its unmaintained `unsafe-libyaml`
  backend, RUSTSEC-2024-0449) with the maintained `serde_norway` fork. YAML
  output of `SwaggerConfig::spec_yaml` / `spec_yaml_response` is unchanged.

## [0.4.0] - 2026-09-15

### Changed

- **Breaking:** requires `armature-core` 0.10 (was `0.9`); its types appear in this crate's API, so the requirement change is breaking here and the minor moves. Part of the `armature-core` 0.10 release train.
- Dependencies bumped to their latest releases: `tokio` 1.52 → 1.53.

