# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support for a Lakeshore 336 Temperature Controller (only temperature reading for all channels) (PR #12).
- This changelog file that will document all notable changes to the project (PR #11).

### Changed

- Updated dependencies to their latest versions (PR #13). This especially includes an update to `measurements` `0.11.1`,
  which now includes support for pressures in Torr and mTorr units.
  This support was used in the Pfeiffer TPG36x gauge driver.
- Add a `SensorError` variant to `InstrumentError` to represent errors related to sensors (PR #12).
- Dropped generics in `LoopbackInterface` and renamed it to `LoopbackInterfaceString`. 
  This interface allows testing of instruments that communicate by sending byte encoded strings 
  with defined line terminators.

## [0.1.0] - 2025-07-30

Release of the first version of `InstrumentRs`.
