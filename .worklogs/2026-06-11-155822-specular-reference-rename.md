# Specular reference rename

## Summary

Renamed stale references to the Specular project in the old machine-checkable specs plan and its worklog. The fixture3 product name, repository name, command, and behavior-suite vocabulary were left unchanged.

## Decisions made

- Kept `fixture3` references because they name this repository and its approval-testing CLI.
- Renamed only the embedded Specular project plan and related filenames.
- Did not change runtime code or fixture behavior.

## Key files for context

- `.plans/2026-05-14-120328-specular-machine-checkable-specs.md`
- `.worklogs/2026-05-14-120443-specular-plan.md`

## Verification

- Local stale-name scan across AQC checkouts.
- Tracked filename scan for previous project names.

## Next steps

- None.
