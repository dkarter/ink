# Changelog

## [0.2.0](https://github.com/dkarter/ink/compare/v0.1.1...v0.2.0) (2026-09-18)


### Features

* **release:** automate artifact publication ([36741bc](https://github.com/dkarter/ink/commit/36741bc4b387da080ca896ab091ee4d55ff67eb0))
* **theme:** add browser and config tooling ([19791d2](https://github.com/dkarter/ink/commit/19791d2e889b1325365a7719fa4dd25f12e0e2c8))


### Bug Fixes

* **deps:** update dependency @fission-ai/openspec to v1.13.1 ([#11](https://github.com/dkarter/ink/issues/11)) ([86ff7b7](https://github.com/dkarter/ink/commit/86ff7b717e5c5780fc9f11e60d4f8928e940c868))
* **deps:** update rust crate syn to v3 ([#9](https://github.com/dkarter/ink/issues/9)) ([1a2f907](https://github.com/dkarter/ink/commit/1a2f907f0ac597f016cecd1f2a06f90614b75cd9))
* **release:** bind artifacts to tagged commit ([1929ec9](https://github.com/dkarter/ink/commit/1929ec90b0d315a64742976ffde8f36237b2985e))

## [0.1.1](https://github.com/dkarter/ink/compare/v0.1.0...v0.1.1) (2026-09-17)


### Bug Fixes

* **release:** exclude generated manifest from formatting ([c163722](https://github.com/dkarter/ink/commit/c16372296719c5ebf9017c8474aa385ec139bdaa))

## 0.1.0 (2026-09-17)


### Features

* bootstrap ink ([4b039bc](https://github.com/dkarter/ink/commit/4b039bc7566432aacf46c38fdb23173415ef485d))
* **cli:** implement interactive prompts ([9c4b8a8](https://github.com/dkarter/ink/commit/9c4b8a85092ec7d5338228e5fc46d95c4ccdb2da))
* complete startup performance checks ([4021157](https://github.com/dkarter/ink/commit/40211577ba41c3c61fc5b48a97b4487b6d0a02f3))
* **config:** implement XDG configuration ([ee81a1a](https://github.com/dkarter/ink/commit/ee81a1a9bd9797cc109e536a1e425588aa8d0da2))
* **editor:** add Vim word operations ([78021c3](https://github.com/dkarter/ink/commit/78021c358ad607c8a6d116a90b26b65a6c03cc41))
* implement Vim editor model ([ccb8463](https://github.com/dkarter/ink/commit/ccb84636657dc73e93a040b6fce7c9968513a40e))
* **prompt:** implement placeholders (RMS-109) ([41af4a3](https://github.com/dkarter/ink/commit/41af4a37f27a0de1af07ea184530366bcbc81bf9))
* **terminal:** implement I/O lifecycle (RMS-106) ([0748d6d](https://github.com/dkarter/ink/commit/0748d6d75944d3b65ff42d41caefa07d47a909f3))
* **theme:** implement semantic palettes ([54df2cb](https://github.com/dkarter/ink/commit/54df2cb12b8d7b88b2870e38d9375ba2a4b3aa27))
* **ui:** implement responsive presentation (RMS-107) ([a460f2b](https://github.com/dkarter/ink/commit/a460f2b83377eab505835eecc0636316dfcbc25d))


### Bug Fixes

* bind releases to approved commit ([4d03ef8](https://github.com/dkarter/ink/commit/4d03ef8e45d5280dd32a40b3b18428c12971a672))
* clarify input cursor logo ([2c77a3b](https://github.com/dkarter/ink/commit/2c77a3b8c724c7b6ae3570b615e3fb3554a87ec9))
* **config:** reject relative XDG config home ([aaba904](https://github.com/dkarter/ink/commit/aaba904b3c821db047172bcb9a71393db5de0050))
* **editor:** place linewise change cursor at EOF ([86498d6](https://github.com/dkarter/ink/commit/86498d64edc08b008a7ddf22564e441ab830e096))
* harden bootstrap foundations ([1f88dda](https://github.com/dkarter/ink/commit/1f88ddad8281553979cd7347f22e53f50287193a))
* include startup benchmark in ci ([5d62c13](https://github.com/dkarter/ink/commit/5d62c134f6652a33dc60416df10616f379dabb9c))
* install rust cache wrapper in ci ([3f2e210](https://github.com/dkarter/ink/commit/3f2e210c9529c651e1584d6109ff140fba9e6b1f))
* install Rust CI components ([89f6a0f](https://github.com/dkarter/ink/commit/89f6a0f5337a406408d0494e4f12ff3223caf501))
* make clean CI installs reproducible ([3bc0f45](https://github.com/dkarter/ink/commit/3bc0f452798b49f3ea1aace4f9d5af72e216bfef))
* normalize Normal mode cursor movement ([2dae47c](https://github.com/dkarter/ink/commit/2dae47c00ecbb0f9144b100bdeff449eb80be4be))
* preserve Vim editor invariants ([91c1ec6](https://github.com/dkarter/ink/commit/91c1ec6b02e114c781012191ffbafc8fb1a76056))
* **prompt:** address RMS-105 review findings ([6b20c1e](https://github.com/dkarter/ink/commit/6b20c1e2720a3fdb02451e1752a383233a73574f))
* **prompt:** address RMS-109 review findings ([bf03468](https://github.com/dkarter/ink/commit/bf0346824b08587f676c46a5abefcccf07e4ec1b))
* round cursor logo ([7e0f28f](https://github.com/dkarter/ink/commit/7e0f28fa6ad121a0edbb85836967b5f284b7d58e))
* strengthen startup behavior checks ([63829f2](https://github.com/dkarter/ink/commit/63829f2bad22ec60a350cf0e071d6ab4b4022337))
* **terminal:** harden I/O lifecycle (RMS-106) ([6a4ebc1](https://github.com/dkarter/ink/commit/6a4ebc145c5d5fd1ff3a30dd928b65c056295b3d))
* **theme:** enforce accessible palette contrast ([d9edead](https://github.com/dkarter/ink/commit/d9edead7f9516567a820bbffc04c0f04289fc623))
* **ui:** address responsive presentation findings (RMS-107) ([06abba7](https://github.com/dkarter/ink/commit/06abba7e4032597455dec873420ba7f7d62096db))
* **ui:** pad textarea mode indicator ([4d44875](https://github.com/dkarter/ink/commit/4d448759437a5a9c78f336a2e55bfb929c0bff6f))
* update website dependencies ([8f0024f](https://github.com/dkarter/ink/commit/8f0024fe7509b959d01821c9c44818864de28be9))

## Changelog

All notable changes to Ink will be documented in this file.

This project follows [Conventional Commits](https://www.conventionalcommits.org/) and uses Release Please to prepare releases.
