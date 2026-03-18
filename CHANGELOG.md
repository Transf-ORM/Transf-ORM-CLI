# Changelog

## [0.2.2](https://github.com/Transf-ORM/Transf-ORM-CLI/compare/cli-v0.2.1...cli-v0.2.2) (2026-03-18)


### Bug Fixes

* remove npm scope and registry-url from Node.js setup ([f4992a9](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/f4992a9489e77bc177be554fed48d105188903c9))

## [0.2.1](https://github.com/Transf-ORM/Transf-ORM-CLI/compare/cli-v0.2.0...cli-v0.2.1) (2026-03-18)


### Bug Fixes

* update npm publish command and add publishConfig to package.json ([76652eb](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/76652ebf042f64484491ed0d3cc3219d30247352))

## [0.2.0](https://github.com/Transf-ORM/Transf-ORM-CLI/compare/cli-v0.1.4...cli-v0.2.0) (2026-03-17)


### Features

* **.github:** push to mirror repository ([6f392c4](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/6f392c4044395ec2b3a1bbe6a712019eea7af136))
* add acceptance criteria section to issue templates and PR template; implement auto-fill workflow for acceptance criteria ([450f321](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/450f3210c0b059afa6bf6d550f80e3db73dca153))
* add dependabot configuration for cargo package updates ([87fb473](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/87fb4738f6b5cdb9c340250dd9538a04f9ee40d0))
* add issue label bot configuration with label aliases ([47ce6f9](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/47ce6f9d4b83e26ef7f801bf1423892633dfb721))
* add issue templates for bug reports, documentation, feature requests, and refactoring ([3c9b27f](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/3c9b27f3b649bcb29108e630764293cd8a654710))
* add new documentation ([419da28](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/419da28e9d3dc058ad03009871ff7ed27ebbfef3))
* add pull request template for consistent contributions ([e3cbb75](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/e3cbb7551b6bdad344b96773687a30f116245da0))
* add security policy document with vulnerability reporting guidelines and best practices ([237671d](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/237671daca26d8dcb688f39a6369e7d7cb5cb700))
* initialize Rust project with Cargo configuration and main function ([028e4b5](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/028e4b581f9a62f3b642f83ca15ce30d30060551))
* update GitHub Actions workflow for wiki synchronization and add maintainers guide ([1a75e08](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/1a75e08e1a8bc4131aa5d5a64ec51ad12f605769))
* **workflow:** add conditional check for repository in mirroring job ([ff0bf90](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/ff0bf90b36b7d8eed37ed03c94502a6bcdc16bbe))
* **workflows:** add build, quality, security, and tests workflows for Rust project ([785ca7e](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/785ca7e654b81ccefabcfd9b7772265c068c5668))


### Bug Fixes

* add GITLEAKS_LICENSE environment variable to gitleaks job in security workflow ([bffe465](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/bffe46595a02ea4e0c96e61e154925c569467429))
* add missing permissions for contents in publish-npm job ([2dafc51](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/2dafc5144b7d2a4a1813d9c6e3970293840e592a))
* add missing scope for npm setup in release workflow ([b7628d8](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/b7628d8756f4757095e1f169b81eed0edbb90411))
* add version.txt for release-please ([75bc91c](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/75bc91c6b40983eedb00b8010b7bd2c043b8f4f0))
* change trigger from pull_request to push for mirroring workflow ([c8fd33c](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/c8fd33cef8b59a52cee156a28f93a96909e35240))
* configure npm for OIDC in release workflow ([7dc41da](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/7dc41dab0319661fb5952327b7552bd98d19a699))
* configure workflow for trusted publishers ([2bc972b](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/2bc972ba388718a5f96f4e169e5968f388151a53))
* fix release config ([0cc245c](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/0cc245c4dc3de3e13f907abfc7f9ba314cb2a353))
* fix release config ([c8bb945](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/c8bb9453d0356815689d1d3c69da79554d555c88))
* fix release config ([823e427](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/823e427de6ea733d8312f42ffe0377ca18b841e9))
* fix release config ([11f9def](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/11f9defccf02eebd10e4f9186193b2068294cff9))
* fix release config ([98db434](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/98db4341d29e2f5495db68ba0b1cdd930b887a06))
* fix release config ([57546a3](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/57546a30731bebd2add83ed91d55506b097cacfe))
* fix release config ([b41d35f](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/b41d35f09924cf6033b634afcc493951c4949ad5))
* fix release config ([b457bb5](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/b457bb58e4c67e61a53ac88a315c603fbae4a444))
* fix release config ([256278d](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/256278dcc3207511eadcdc32f520341f4c321718))
* fix release config ([37356b1](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/37356b14a680b6c14ca3c6cb0a026df99fb842df))
* fix release config ([518d0e2](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/518d0e2785cb77b3cf348e6bff080698095d3159))
* fix release config ([03d0f72](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/03d0f72e78416639e019e770da12418a649e771c))
* fix release config ([a670ab9](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/a670ab96ffa1f7bb050af0ac157549efa588fc60))
* fix release config ([1ef5c80](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/1ef5c8057442141e98ebd184477fb74b9cf14ff1))
* fix release config ([cf3da1d](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/cf3da1d51860e54032d41aeab18d08df379fc13f))
* fix release config ([98aca34](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/98aca3423218fa214cf66ea0581e25beb141ff7b))
* fix release config ([303a4ae](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/303a4aea316ab63c76d774e9e9dcf8356054e9ba))
* fix release config ([0eaeb03](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/0eaeb03c059e9f877e851fe2b7529b84cd164dff))
* fix release config ([2efa774](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/2efa774d1313219fecdd264364fe674bb6f7f998))
* **release-please:** update release configuration and fix welcome message ([23a1d19](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/23a1d1932346fd7e70698bd36e6f95a1b46eb42c))
* remove redundant npm configuration step in release workflow ([f2378c0](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/f2378c054f36cb718a8c637038f7e439e4e1f54a))
* remove release-please manifest files ([723fe59](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/723fe5958554785eb3d09e774fe7cc7cfe930266))
* remove release-please manifest files ([fd53921](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/fd539218de2d803b6423f211c4263891dce39d24))
* test release after manifest update ([98d9018](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/98d901847bfc290c545a4af41ff2e0eeb80c84b2))
* trigger release-please ([425c252](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/425c2526e1a20b1f79bcfa8d4d393fde3fe44489))
* trigger release-please ([a957015](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/a957015de6655d54a532c3a47a034bcdedb2bb5d))
* trigger release-please ([e265af8](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/e265af8f05a38a2b6cfa2eee82c754a59ef6555b))
* trigger release-please ([16b39ee](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/16b39ee9dc7ff8a3e1d2fb0e16b4fdee262e1096))
* trigger release-please ([112516a](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/112516ae4ec35c5ea4187902376ef22779b1b80c))
* update repository name from 'nitrached/Cod3x' to 'Transf-ORM/Transf-ORM-CLI' in workflow files and Cargo configuration ([696b1a3](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/696b1a3f6b2e5c8799c5649a6dea79a578e473d2))

## [0.1.4](https://github.com/Transf-ORM/Transf-ORM-CLI/compare/cli-v0.1.3...cli-v0.1.4) (2026-03-17)


### Bug Fixes

* add missing scope for npm setup in release workflow ([b7628d8](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/b7628d8756f4757095e1f169b81eed0edbb90411))

## [0.1.3](https://github.com/Transf-ORM/Transf-ORM-CLI/compare/cli-v0.1.2...cli-v0.1.3) (2026-03-17)


### Bug Fixes

* add missing permissions for contents in publish-npm job ([2dafc51](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/2dafc5144b7d2a4a1813d9c6e3970293840e592a))

## [0.1.2](https://github.com/Transf-ORM/Transf-ORM-CLI/compare/cli-v0.1.1...cli-v0.1.2) (2026-03-17)


### Bug Fixes

* remove redundant npm configuration step in release workflow ([f2378c0](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/f2378c054f36cb718a8c637038f7e439e4e1f54a))

## [0.1.1](https://github.com/Transf-ORM/Transf-ORM-CLI/compare/cli-v0.1.0...cli-v0.1.1) (2026-03-17)


### Bug Fixes

* configure npm for OIDC in release workflow ([7dc41da](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/7dc41dab0319661fb5952327b7552bd98d19a699))

## [0.1.0](https://github.com/Transf-ORM/Transf-ORM-CLI/compare/cli-v0.0.1...cli-v0.1.0) (2026-03-17)


### Features

* **.github:** push to mirror repository ([6f392c4](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/6f392c4044395ec2b3a1bbe6a712019eea7af136))
* add acceptance criteria section to issue templates and PR template; implement auto-fill workflow for acceptance criteria ([450f321](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/450f3210c0b059afa6bf6d550f80e3db73dca153))
* add dependabot configuration for cargo package updates ([87fb473](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/87fb4738f6b5cdb9c340250dd9538a04f9ee40d0))
* add issue label bot configuration with label aliases ([47ce6f9](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/47ce6f9d4b83e26ef7f801bf1423892633dfb721))
* add issue templates for bug reports, documentation, feature requests, and refactoring ([3c9b27f](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/3c9b27f3b649bcb29108e630764293cd8a654710))
* add new documentation ([419da28](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/419da28e9d3dc058ad03009871ff7ed27ebbfef3))
* add pull request template for consistent contributions ([e3cbb75](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/e3cbb7551b6bdad344b96773687a30f116245da0))
* add security policy document with vulnerability reporting guidelines and best practices ([237671d](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/237671daca26d8dcb688f39a6369e7d7cb5cb700))
* initialize Rust project with Cargo configuration and main function ([028e4b5](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/028e4b581f9a62f3b642f83ca15ce30d30060551))
* update GitHub Actions workflow for wiki synchronization and add maintainers guide ([1a75e08](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/1a75e08e1a8bc4131aa5d5a64ec51ad12f605769))
* **workflow:** add conditional check for repository in mirroring job ([ff0bf90](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/ff0bf90b36b7d8eed37ed03c94502a6bcdc16bbe))
* **workflows:** add build, quality, security, and tests workflows for Rust project ([785ca7e](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/785ca7e654b81ccefabcfd9b7772265c068c5668))


### Bug Fixes

* add GITLEAKS_LICENSE environment variable to gitleaks job in security workflow ([bffe465](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/bffe46595a02ea4e0c96e61e154925c569467429))
* add version.txt for release-please ([75bc91c](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/75bc91c6b40983eedb00b8010b7bd2c043b8f4f0))
* change trigger from pull_request to push for mirroring workflow ([c8fd33c](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/c8fd33cef8b59a52cee156a28f93a96909e35240))
* configure workflow for trusted publishers ([2bc972b](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/2bc972ba388718a5f96f4e169e5968f388151a53))
* fix release config ([0cc245c](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/0cc245c4dc3de3e13f907abfc7f9ba314cb2a353))
* fix release config ([c8bb945](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/c8bb9453d0356815689d1d3c69da79554d555c88))
* fix release config ([823e427](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/823e427de6ea733d8312f42ffe0377ca18b841e9))
* fix release config ([11f9def](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/11f9defccf02eebd10e4f9186193b2068294cff9))
* fix release config ([98db434](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/98db4341d29e2f5495db68ba0b1cdd930b887a06))
* fix release config ([57546a3](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/57546a30731bebd2add83ed91d55506b097cacfe))
* fix release config ([b41d35f](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/b41d35f09924cf6033b634afcc493951c4949ad5))
* fix release config ([b457bb5](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/b457bb58e4c67e61a53ac88a315c603fbae4a444))
* fix release config ([256278d](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/256278dcc3207511eadcdc32f520341f4c321718))
* fix release config ([37356b1](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/37356b14a680b6c14ca3c6cb0a026df99fb842df))
* fix release config ([518d0e2](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/518d0e2785cb77b3cf348e6bff080698095d3159))
* fix release config ([03d0f72](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/03d0f72e78416639e019e770da12418a649e771c))
* fix release config ([a670ab9](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/a670ab96ffa1f7bb050af0ac157549efa588fc60))
* fix release config ([1ef5c80](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/1ef5c8057442141e98ebd184477fb74b9cf14ff1))
* fix release config ([cf3da1d](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/cf3da1d51860e54032d41aeab18d08df379fc13f))
* fix release config ([98aca34](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/98aca3423218fa214cf66ea0581e25beb141ff7b))
* fix release config ([303a4ae](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/303a4aea316ab63c76d774e9e9dcf8356054e9ba))
* fix release config ([0eaeb03](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/0eaeb03c059e9f877e851fe2b7529b84cd164dff))
* fix release config ([2efa774](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/2efa774d1313219fecdd264364fe674bb6f7f998))
* **release-please:** update release configuration and fix welcome message ([23a1d19](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/23a1d1932346fd7e70698bd36e6f95a1b46eb42c))
* remove release-please manifest files ([723fe59](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/723fe5958554785eb3d09e774fe7cc7cfe930266))
* remove release-please manifest files ([fd53921](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/fd539218de2d803b6423f211c4263891dce39d24))
* test release after manifest update ([98d9018](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/98d901847bfc290c545a4af41ff2e0eeb80c84b2))
* trigger release-please ([425c252](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/425c2526e1a20b1f79bcfa8d4d393fde3fe44489))
* trigger release-please ([a957015](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/a957015de6655d54a532c3a47a034bcdedb2bb5d))
* trigger release-please ([e265af8](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/e265af8f05a38a2b6cfa2eee82c754a59ef6555b))
* trigger release-please ([16b39ee](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/16b39ee9dc7ff8a3e1d2fb0e16b4fdee262e1096))
* trigger release-please ([112516a](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/112516ae4ec35c5ea4187902376ef22779b1b80c))
* update repository name from 'nitrached/Cod3x' to 'Transf-ORM/Transf-ORM-CLI' in workflow files and Cargo configuration ([696b1a3](https://github.com/Transf-ORM/Transf-ORM-CLI/commit/696b1a3f6b2e5c8799c5649a6dea79a578e473d2))
