# Changelog

## [0.7.4](https://github.com/bnusunny/konductor/compare/v0.7.3...v0.7.4) (2026-03-22)


### Bug Fixes

* hooks field must be a map keyed by event type ([#43](https://github.com/bnusunny/konductor/issues/43)) ([7d873d3](https://github.com/bnusunny/konductor/commit/7d873d3b16f45d0bb94465a27fd31c05cc2428bd))

## [0.7.3](https://github.com/bnusunny/konductor/compare/v0.7.2...v0.7.3) (2026-03-22)


### Bug Fixes

* inline hooks config in agent definitions ([#41](https://github.com/bnusunny/konductor/issues/41)) ([83dfd58](https://github.com/bnusunny/konductor/commit/83dfd58ee1e2e0831765658cd11cf75073fdcf23))

## [0.7.2](https://github.com/bnusunny/konductor/compare/v0.7.1...v0.7.2) (2026-03-22)


### Bug Fixes

* allow state transition from shipped to initialized ([#39](https://github.com/bnusunny/konductor/issues/39)) ([8bef9aa](https://github.com/bnusunny/konductor/commit/8bef9aaf79ce836e0d2596494c0194c2d2ee2fe3))

## [0.7.1](https://github.com/bnusunny/konductor/compare/v0.7.0...v0.7.1) (2026-03-22)


### Bug Fixes

* wire hooks config to all agent definitions ([#37](https://github.com/bnusunny/konductor/issues/37)) ([e157091](https://github.com/bnusunny/konductor/commit/e15709128ac93508fe41614c10452fbd8dec46d6))

## [0.7.0](https://github.com/bnusunny/konductor/compare/v0.6.0...v0.7.0) (2026-03-22)


### Features

* npm install now installs agents, skills, and hooks to ~/.kiro/ ([#36](https://github.com/bnusunny/konductor/issues/36)) ([80e1c87](https://github.com/bnusunny/konductor/commit/80e1c874d2f5bc96ec66cff8549a6b6138692bb1))


### Bug Fixes

* trigger npm-publish workflow after binaries are built ([#34](https://github.com/bnusunny/konductor/issues/34)) ([4e66971](https://github.com/bnusunny/konductor/commit/4e669710c804339949caf24bb1fe712d80b61478))

## [0.6.0](https://github.com/bnusunny/konductor/compare/v0.5.1...v0.6.0) (2026-03-22)


### Features

* add state_init MCP tool, remove LLM-generated TOML from init skill ([#32](https://github.com/bnusunny/konductor/issues/32)) ([362159f](https://github.com/bnusunny/konductor/commit/362159ff58abd3568d9371d7a31aa76be207bf91))

## [0.5.1](https://github.com/bnusunny/konductor/compare/v0.5.0...v0.5.1) (2026-03-22)


### Bug Fixes

* add [@konductor](https://github.com/konductor) to agent tools array for MCP tool availability ([#31](https://github.com/bnusunny/konductor/issues/31)) ([de8d47a](https://github.com/bnusunny/konductor/commit/de8d47ad339ccd197813badf57489cda2f6fe63f))
* **ci:** add environment for npm trusted publishing ([83f1400](https://github.com/bnusunny/konductor/commit/83f14005c7d3e4b21005f3e25d64389ae60e248d))
* **ci:** install npm@latest for OIDC trusted publishing support ([4a3b450](https://github.com/bnusunny/konductor/commit/4a3b45005247e293eb5a538226000fded6514c7e))
* **ci:** use npm OIDC publish without registry-url to avoid dummy token ([6a67f35](https://github.com/bnusunny/konductor/commit/6a67f35336a87b17bd1a53e24f86d19250c25368))

## [0.5.0](https://github.com/bnusunny/konductor/compare/v0.4.1...v0.5.0) (2026-03-22)


### Features

* add npm distribution with trusted publishing ([9402fd0](https://github.com/bnusunny/konductor/commit/9402fd09ab5ba39de147a358c9dbac5c1788353b))
* add npm distribution with trusted publishing ([#29](https://github.com/bnusunny/konductor/issues/29)) ([d1c04cc](https://github.com/bnusunny/konductor/commit/d1c04cc92299425d6461cc682ef671c8d3de4412))

## [0.4.1](https://github.com/bnusunny/konductor/compare/v0.4.0...v0.4.1) (2026-03-22)


### Bug Fixes

* **ci:** add workflow_dispatch trigger to build-release workflow ([56989bc](https://github.com/bnusunny/konductor/commit/56989bcaa694cd5de1b6de6758f254ba9d214586))
* **ci:** chain build-release from release-please via workflow_dispatch ([0e69e89](https://github.com/bnusunny/konductor/commit/0e69e892d89f6e49ffb86b796d4af1ee37130937))
* **ci:** checkout release tag in build-release workflow ([d371885](https://github.com/bnusunny/konductor/commit/d37188586a329e41014d90d2ee1fc7ce2d907787))

## [0.4.0](https://github.com/bnusunny/konductor/compare/v0.3.0...v0.4.0) (2026-03-22)


### Features

* add integration tests, E2E pipeline, benchmarks, and self-improvement loop ([#23](https://github.com/bnusunny/konductor/issues/23)) ([b29cc98](https://github.com/bnusunny/konductor/commit/b29cc98102cb0cb52c94477675a3c1780384237f))
* multi-project benchmarks, SAM deployment pipeline, self-improvement loop ([1ff757d](https://github.com/bnusunny/konductor/commit/1ff757d8d36882ff8e4bd8ab4a9f8bd10a0c68cd))


### Bug Fixes

* use correct hook binary path for local installs ([#26](https://github.com/bnusunny/konductor/issues/26)) ([df1f656](https://github.com/bnusunny/konductor/commit/df1f656fba95200ca4fc4292a698f09a1ed08af9))

## [0.3.0](https://github.com/bnusunny/konductor/compare/v0.2.1...v0.3.0) (2026-03-21)


### Features

* konductor v0.3.0 — CLI hardening, install/distribution, docs ([#21](https://github.com/bnusunny/konductor/issues/21)) ([9c5d5df](https://github.com/bnusunny/konductor/commit/9c5d5df83de0fbf014d6df166ccfd3aa082e6f16))


### Bug Fixes

* **ci:** ensure build job runs on tag-triggered workflows ([#16](https://github.com/bnusunny/konductor/issues/16)) ([da9d948](https://github.com/bnusunny/konductor/commit/da9d948b8851d3901d1fa1b9d99b4e772ee70549))

## [0.2.1](https://github.com/bnusunny/konductor/compare/v0.2.1...v0.2.1) (2026-03-21)


### Bug Fixes

* **ci:** ensure build job runs on tag-triggered workflows ([#16](https://github.com/bnusunny/konductor/issues/16)) ([da9d948](https://github.com/bnusunny/konductor/commit/da9d948b8851d3901d1fa1b9d99b4e772ee70549))

## [0.2.1](https://github.com/bnusunny/konductor/compare/v0.2.0...v0.2.1) (2026-03-21)


### Bug Fixes

* **ci:** align release binary names with install.sh expectations ([f7173e6](https://github.com/bnusunny/konductor/commit/f7173e601829f8ada07ca92e57736b753fc19332))
* **ci:** align release binary names with install.sh expectations ([b462cbc](https://github.com/bnusunny/konductor/commit/b462cbcc4d2f3d8f08fc1d8ddf108a4b68fb6f6f))

## [0.2.0](https://github.com/bnusunny/konductor/compare/v0.1.0...v0.2.0) (2026-03-21)


### Features

* add unified konductor binary with MCP server and hook subcommands ([7f11640](https://github.com/bnusunny/konductor/commit/7f116405e231cf00227d58aba5619593f256b1bb))
* read release-as version from version.txt ([c133580](https://github.com/bnusunny/konductor/commit/c133580d0e21022766f14d1e51d20e617dbf2f40))
* read release-as version from version.txt ([b1b81e7](https://github.com/bnusunny/konductor/commit/b1b81e76ebbe18b6a13aa100ec4f99a6bdec5046))
* unified konductor binary with MCP server and hook subcommands ([1de12ef](https://github.com/bnusunny/konductor/commit/1de12ef5847d82a027bfa4fe1e87d664fbe67959))

## [0.1.0](https://github.com/bnusunny/konductor/compare/v0.1.0...v0.1.0) (2026-03-20)


### Features

* add all worker agent configs ([7f9e5d8](https://github.com/bnusunny/konductor/commit/7f9e5d87f1e7a4531f02d783eb4a376cfdfdc8e2))
* add hook system with Rust binary for file tracking and safety guardrails ([0fa8374](https://github.com/bnusunny/konductor/commit/0fa8374fbf3517f24f1155b96dda2216c6f7c822))
* add installer script ([2ac5a6e](https://github.com/bnusunny/konductor/commit/2ac5a6e5af0c37d4227f519c46367fecb9ccfea4))
* add konductor-exec skill with execution and TDD references ([830a287](https://github.com/bnusunny/konductor/commit/830a287cfae3b82e7f8316c538a2d590dc432dbf))
* add konductor-init skill with questioning reference ([ed93aa2](https://github.com/bnusunny/konductor/commit/ed93aa2832fe796874c8e6c463ff9b6505306ef1))
* add konductor-next super-skill ([d284c76](https://github.com/bnusunny/konductor/commit/d284c76a7116466dcc98d34b2723c7adba481cb7))
* add konductor-plan skill with planning and verification references ([4d0b074](https://github.com/bnusunny/konductor/commit/4d0b0742c813621fb66a53ced8cf4e033cb5a787))
* add konductor-verify skill ([694f17f](https://github.com/bnusunny/konductor/commit/694f17f6cf29cba12a3490dc9aa3eafacd235f0d))
* add one-line installer for GitHub Pages ([03e3dc7](https://github.com/bnusunny/konductor/commit/03e3dc76da49c34545aaf975777a768a0f0d7791))
* add status, ship, discuss, and map-codebase skills ([12fbdcd](https://github.com/bnusunny/konductor/commit/12fbdcd4030f1726d1380885b6b61115d6b8f225))
* repository scaffold with orchestrator agent config ([4dfdfe2](https://github.com/bnusunny/konductor/commit/4dfdfe275c35ab82dca610326238ba04038f50bf))


### Bug Fixes

* default to global scope and make hook binary download optional ([b67788c](https://github.com/bnusunny/konductor/commit/b67788c7e513b33b2d05c018bb050764ddc1f04d))
* replace Material icon in button with arrow character ([17f144b](https://github.com/bnusunny/konductor/commit/17f144b9772c32f1afffbd379899958710d51410))
* use standard markdown on homepage instead of Material icons ([82e44b1](https://github.com/bnusunny/konductor/commit/82e44b1413641b581eb0ad6f801de9737d3afd42))

## 0.1.0 (2026-03-20)


### Features

* add all worker agent configs ([7f9e5d8](https://github.com/bnusunny/konductor/commit/7f9e5d87f1e7a4531f02d783eb4a376cfdfdc8e2))
* add hook system with Rust binary for file tracking and safety guardrails ([0fa8374](https://github.com/bnusunny/konductor/commit/0fa8374fbf3517f24f1155b96dda2216c6f7c822))
* add installer script ([2ac5a6e](https://github.com/bnusunny/konductor/commit/2ac5a6e5af0c37d4227f519c46367fecb9ccfea4))
* add konductor-exec skill with execution and TDD references ([830a287](https://github.com/bnusunny/konductor/commit/830a287cfae3b82e7f8316c538a2d590dc432dbf))
* add konductor-init skill with questioning reference ([ed93aa2](https://github.com/bnusunny/konductor/commit/ed93aa2832fe796874c8e6c463ff9b6505306ef1))
* add konductor-next super-skill ([d284c76](https://github.com/bnusunny/konductor/commit/d284c76a7116466dcc98d34b2723c7adba481cb7))
* add konductor-plan skill with planning and verification references ([4d0b074](https://github.com/bnusunny/konductor/commit/4d0b0742c813621fb66a53ced8cf4e033cb5a787))
* add konductor-verify skill ([694f17f](https://github.com/bnusunny/konductor/commit/694f17f6cf29cba12a3490dc9aa3eafacd235f0d))
* add one-line installer for GitHub Pages ([03e3dc7](https://github.com/bnusunny/konductor/commit/03e3dc76da49c34545aaf975777a768a0f0d7791))
* add status, ship, discuss, and map-codebase skills ([12fbdcd](https://github.com/bnusunny/konductor/commit/12fbdcd4030f1726d1380885b6b61115d6b8f225))
* repository scaffold with orchestrator agent config ([4dfdfe2](https://github.com/bnusunny/konductor/commit/4dfdfe275c35ab82dca610326238ba04038f50bf))


### Bug Fixes

* default to global scope and make hook binary download optional ([b67788c](https://github.com/bnusunny/konductor/commit/b67788c7e513b33b2d05c018bb050764ddc1f04d))
* replace Material icon in button with arrow character ([17f144b](https://github.com/bnusunny/konductor/commit/17f144b9772c32f1afffbd379899958710d51410))
* use standard markdown on homepage instead of Material icons ([82e44b1](https://github.com/bnusunny/konductor/commit/82e44b1413641b581eb0ad6f801de9737d3afd42))
