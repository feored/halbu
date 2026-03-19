# Changelog

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
