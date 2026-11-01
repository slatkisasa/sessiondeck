# Issue tracker: GitHub

Issues and specs live in slatkisasa/sessiondeck on GitHub.
Use the gh CLI from this repository.

## Operations

- Publish a ticket: gh issue create --title "..." --body-file <file>.
- Read a ticket: gh issue view <number> --json number,title,body,labels,comments.
- List tickets: gh issue list --state open with appropriate label filters.
- Comment: gh issue comment <number> --body-file <file>.
- Change labels: gh issue edit <number> --add-label "..." or --remove-label "...".
- Close: post the explanation as a comment, then gh issue close <number>.

Before creating the first issue in a session, verify the target with
gh repo view --json nameWithOwner. It must be slatkisasa/sessiondeck.

Use native GitHub sub-issues for parent/child relationships and native
issue dependencies for blockers. If unavailable, put "Part of #<parent>"
or "Blocked by: #<number>" at the top of the child body.
A ticket is unblocked when every blocker is closed.

## Pull requests as a triage surface

PRs as a request surface: no.

## Wayfinding

Keep the map in one issue labelled wayfinder:map.
Link child tickets as sub-issues and label them wayfinder:<type>,
where type is research, prototype, grilling, or task.

Choose the first open child in map order with no open blockers or assignee.
Claim it by assigning it to the driving developer.
Resolve it by commenting with the answer, closing it, and appending a
summary and link to the map's Decisions-so-far.
