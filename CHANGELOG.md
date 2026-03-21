# Changelog

## 0.3.0



- **Breaking**: Marked `CompatibilityCode`, `ValidationCode`, `IssueKind`, and `FormatId` as `#[non_exhaustive]` to make room for future variants without repeated semver breaks.
- Changing `mercenary.id` between `0` (no mercenary hired) and nonzero (mercenary hired) now reports a blocking compatibility issue under `CompatibilityChecks::Enforce`, because Halbu does not yet rewrite the mercenary item subsection inside the raw item tail.

- Fixed mercenary encoding to zero the full mercenary header block when no mercenary is hired.
- Added parse and validation coverage for ghost mercenary data on non-hired mercenaries.
- Added regression tests around mercenary hire-state handling.
- Removed an over-broad quest-state validation check on prologue/completion markers.

## 0.2.3

- Fixed a bugged difficulty unlock warning by accepting `CompletedBefore` as a quest completion marker.
- Reworked progression validation to use a difficulty floor instead of requiring an exact normalized value.
- Added a real high level save as a regression fixture to keep the save parse and validation checks clean.

## 0.2.2

- Removed unused `log` dependency.

## 0.2.1

- Added optional save validation reports, separate from encoding.

## 0.2.0

- Added RotW support.
- Added v99/v105 coverage.
- Added Warlock support and moved named skills to a separate module.
- Split format, edition, and expansion concepts more cleanly.
- Added save summaries.
- Exposed checksum metadata in parsed saves.
- Added compatibility rules and forced encode handling.
- Added a best-effort save game edition hint.
- Added examples.
