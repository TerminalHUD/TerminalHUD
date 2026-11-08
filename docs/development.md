# Ratpack contributor workflow

Humans and agents share one task, branch, PR, and review process. Linear owns
task state, ownership, and dependencies; GitHub owns code and review evidence;
repository documents own requirements and design. Use the adopted project
workflow and explicit user decisions; upstream changes require deliberate adoption.

## Roles and authority

| Role | General agent name | Responsibility and handoff |
| --- | --- | --- |
| Coordinator | `crd-rat` | Plan product work, prioritize tasks, assign owners, resolve dependencies, and collect results |
| Implementation | `imp-rat` | Build one assigned change with tests/docs; hand off a commit or PR and validation |
| Independent review | `rev-rat` | Evaluate the exact proposed revision; return actionable findings or no findings, with checks and limitations |
| Cleanup | `cln-rat` | Audit a bounded area for maintainability problems; report findings or implement scoped fixes |
| Workflow | `wfl-rat` | Maintain contributor guidance and tracker/repository setup; report changes, verified behavior, and setup gaps |

The coordinator is the persistent project conversation by default. It delegates
substantial implementation when authorized and supported, and may implement
bounded work itself. Cleanup and workflow are optional support, not permanent
staffing requirements. Every assignment names the outcome, acceptance criteria,
owned paths, dependencies/contracts, checks, and handoff destination.

Only the integration owner recorded in `ratpack.json` merges PRs, one at a time.
The coordinator normally fills that responsibility; another assignment requires
an explicit owner transfer. The user retains product and release authority.
To transfer integration, the user names a replacement, the old owner stops and
hands off the queue, and the record changes before the new owner merges. If the
old owner is unavailable, confirm it will not merge and reconstruct the queue
from GitHub. Conversation names and role selection do not confer authority.

Authority is repository-specific: a conversation may coordinate one project
and contribute to another without carrying merge rights or project context
between them. Worker assignments do not permit scope expansion or recursive
spawning; additional spawning authority must be explicitly delegated.

### Staffing and contributors

There is no fixed worker count. The coordinator proposes a useful concurrent
implementation count and task split; the human approves or chooses another.
Record the approved count in `max_implementation_agents` and spawning permission
in `implementation_delegation_authorized`. A null count is unapproved, not
unlimited. Spawning implementors requires permission and a positive approved
count. Reuse approval until changed; request an increase when useful and wait
for an answer before exceeding existing authorization.

Use fewer workers for overlapping tasks and respect actual host capacity.
Human-created implementation peers and cleanup/custom roles that edit repository
content consume the same worker capacity. Review/support spawning requires its
own agreed scope; the implementation limit does not authorize unlimited support.

The human may create a separate conversation for any role. Register its actual
owner and handoff route, have it read the project instructions and role brief,
and avoid duplicate assignments. Peers remain human-accessible; they need not
become subagents. Humans do not need agent session bookkeeping.

Use the role table's general names for reusable agents; projects may choose
other names. Reuse suitable general agents across assignments. Add a task suffix
only when it distinguishes concurrent or specialized workers, such as
`imp-rat-storage`. Adapt punctuation to host constraints: `imp_rat_storage` when
internal names require underscores. Each assignment still has its own bounded
brief and ownership record.

### Custom roles and native profiles

The human may add, change, or retire a role through the coordinator at any time.
Record its purpose, allowed work, expected outputs/evidence, and handoff under
`custom_roles`. Reconcile active assignments before changing or retiring a role.
A definition grants no new authority and does not require a permanent agent or
another skill; use the role procedure when maintaining a native definition.

Bootstrap supplies `ratpack_coordinator`, `ratpack_implementor`,
`ratpack_reviewer`, `ratpack_cleanup`, and `ratpack_workflow` profiles in
`.codex/agents/`. Profiles select these role briefs; skills provide procedures.
Profiles inherit the parent's model, sandbox/approvals, and MCP access. Behavioral
read-only instructions are not a separate sandbox. Verify discovery in a trusted,
fresh/reloaded session before claiming activation; another host may use the same
briefs without native discovery. Renaming a conversation does not activate a profile.

## Task to merge

For normal repository changes; release/hotfix routing is specified below.

1. Create or reuse a bounded tracker task for actual work, not hypothetical
   discussion. Record scope, acceptance evidence, owner or pending assignment,
   dependencies, start condition, next action, and relevant design links.
2. Assign one owner and an independent reviewer. Agree shared interfaces before
   parallel work; sequence overlapping schemas, headers, and build files.
3. Branch from current `origin/devel` using `feature/`, `fix/`, `docs/`, or
   `chore/` plus the real task ID and short description when available.
   Each implementation task gets its own branch/worktree; humans may use a
   normal checkout. Inspect status, branches, and worktrees first. Never switch,
   reset, stash, clean, or stage unrelated work in another owner's checkout.
4. Implement one reviewable change with applicable tests/docs. Open a PR into
   `devel`, link the task, and report checks/results and gaps.
5. A different human or agent reviews the exact head for requirements,
   correctness, regressions, and test evidence, including coordinator-authored
   changes. Record severity, location, reasoning/evidence for findings; separate
   blockers from optional polish. The reviewer does not edit the author's branch.
   Experiments that modify files/build outputs use an isolated copy. Re-review
   affected changes after updates; old review does not approve a new revision.
6. The integration owner verifies prerequisites, current review, required GitHub
   approvals, and CI, then squash merges. Agent review does not manufacture a
   GitHub approval. Mark repository-change tasks Done only after accepted merge.

Merge prerequisites before dependents. Synchronize owned branches with current
`devel` by merge or transplanting dependent commits; retain only the intended
diff and repeat affected checks/review. Keep one implementation task active per
worker. Occupied worktrees remain until their owner hands them off; remove them
only when no remaining work depends on them.

## Requests and ordering

Capture actual requests as bounded tracker tasks for every role, including
audits, research, docs, and custom work; hypothetical discussion creates none.
Record relative requests as explicit start conditions in the task.

Use conversation and project context to identify the owning project. Update an
existing task for a clarification or related scope; capture independent work
as a new or existing task in that project's queue. Resolve genuine project
ambiguity before tracker writes or dispatch. Route work outside your project or
assigned role to the appropriate coordinator without taking over its authority.

An unscheduled request is capture and triage, not implicit cancellation or
preemption. Continue active assignments; schedule new work through existing
priorities, prerequisites, and capacity. Clarifications needed to finish the
current task belong on that task. Reprioritize when the human clearly requests
it, coordinating a safe handoff where needed. Acknowledge the captured task and
next action; creating a ticket is not dispatch or completion.

| Request | Start condition |
| --- | --- |
| No timing specified | Capture/update the project task and schedule within existing priorities, dependencies, and capacity |
| Now | Next eligible opportunity within authority, ownership, dependencies, and capacity |
| Next / when free | Next suitable slot with prerequisites intact |
| After named tasks | Those exact IDs and their accepted deliverables |
| After current tickets | Fixed snapshot of In Progress/In Review IDs at request time, unless the user names another batch |

Acknowledge the snapshot IDs; future tickets do not join it. If the batch is
empty, say so and treat the condition as met. Resolve an unclear batch before
dispatch. Use tracker blocking relations, exclude the target itself, and check
for cycles: required review cannot wait for its own implementation to be Done.
Reassess canceled/replaced prerequisites and record the decision or replacement.

Deferred work stays Backlog until its condition is met, then Ready; mark it
In Progress only when it starts. The whole task is deferred, including its audit
phase, unless the human explicitly separates audit-now from fixes-later. Urgency
does not bypass prerequisites or justify disrupting another owner's checkout;
coordinate a safe handoff for preemption and report any blocker/next action.

Report-only work finishes when its agreed artifact is delivered and accepted
by the recipient, with evidence linked in the task. A worker's return alone
is not acceptance. Reuse coherent tasks rather than creating a ticket per tiny
finding. Recheck the queue on new requests, handoffs, merges, and resume; continue
authorized ready work while active. Tracker relations do not wake a stopped
coordinator; calendar execution needs separately requested, verified automation.

## Communication and handoffs

Use readable language with normal spaces and line breaks; preserve exact
identifiers, commands, paths, and quoted evidence. Link authoritative records
rather than repeatedly forwarding summaries or whole conversations. Resolve
truncated or ambiguous messages against their source before acting.

A handoff records task, branch/worktree, exact commit/PR, checks/results,
limitations/open findings, and next owner/action. Include agent identity and
session route in task notes when needed. Ownership/authority transfers require
recipient acknowledgment before conflicting work starts. Distinguish requests,
acknowledgments, and verified completion; summaries are not proof of passed checks.

## Linear

Reuse existing teams/projects. Start with one team and one project per bounded
milestone; track current work and near-term dependencies rather than the entire
historical roadmap. A code-change task normally corresponds to one PR.

| Status | Meaning |
| --- | --- |
| Backlog | Captured work not ready or selected |
| Ready | Scope, acceptance criteria, dependencies, and start condition permit work |
| In Progress | An owner is doing the task |
| In Review | A ready PR awaits review/integration, or a report awaits acceptance |
| Done | Accepted deliverable complete; repository changes satisfy Task to merge |
| Canceled | Abandoned with a reason or replacement |

Use the team's available statuses with these meanings: Ready is Unstarted;
In Progress/In Review are Started. Start with Bug, Feature, Improvement, and
Documentation labels; docs use the same states. Feature-required docs belong
in that feature's acceptance. Prefer owners, priorities, projects, and blocking
relations over redundant labels. Extra labels and cycles are optional.

Configure supported GitHub automation targeting `devel`: draft → In Progress,
ready PR → In Review, merge → Done. Verify it with a real linked PR; state
changes are not independent approval. If access/automation is unavailable,
link evidence and update manually, track pending setup, and keep work moving.

## Branches and releases

| Change | Target | Merge method |
| --- | --- | --- |
| Feature, fix, docs, chore | `devel` | Squash |
| Release from `devel` | `main` | Merge commit |
| Hotfix from released `main` | `main` | Squash, then merge `main` into `devel` |

Use `devel` as default branch. Enable squash/merge commits and disable rebase
merges. Never squash between long-lived branches, rebase shared `main`/`devel`,
or force-push them. The user approves release scope/version; the integration
owner verifies evidence, merges the release PR, and tags its `main` commit.
Merge `main` back into `devel` after release. Track release completion separately
from implementation; temporary release branches may stabilize a release.

Protect both branches with required PR review and CI where available; prohibit
direct/force pushes. Disclose unavailable protections and apply the checklist
manually; visibility/billing changes need user approval. A branch is not proof
of deployment or product readiness.

For a new repository, a reviewed baseline may initialize `main` and `devel` at
the same commit. Existing repositories need a reviewed migration preserving
history and occupied work; never relabel arbitrary code as a released version.

## TerminalHUD adoption

Adopted from `ratpack-workflow/ratpack` revision
`86a12eb12bc1fcd5b4bb3e722056c21a4f226aac` using the installed `rp-bootstrap`
skill. The installed skill, templates, workflow, and helper scripts were checked
against the upstream Git blob hashes at that revision. The lock records the
helper's generated baseline with supplied provenance; project customization is
intentional and must be preserved during upgrades.

The product requirements and architecture decisions remain authoritative.
Implementation has not started. Repository checks cover contributor
configuration, the five native role profile files, local Markdown targets, and
whitespace. Add Cargo and Windows validation with implementation.

### Project setup and bootstrap handoff

- Tracker: team `Terminalhud` (`TER`) through `linear_terminalhud`; project
  [TerminalHUD Windows MVP](https://linear.app/terminalhud/project/terminalhud-windows-mvp-b25fa1b76f71).
  [TER-5](https://linear.app/terminalhud/issue/TER-5/adopt-ratpack-contributor-workflow-and-repository-setup)
  owns adoption evidence, review, and the next action. Task state belongs there.
- The user assigned this conversation as coordinator and implementor. Integration
  owner is `crd-rat (TerminalHUD coordinator conversation)` in `ratpack.json`,
  following the coordinator's normal integration responsibility. The user retains
  product and release authority. Direct implementation by this coordinator is
  allowed; implementation delegation remains disabled with an unapproved
  (`null`) worker count. Additional review/support agents require authorization.
- The team's `Todo` state maps to Ratpack Ready. Backlog, In Progress, In Review,
  Done, and Canceled already exist, along with Bug, Feature, Improvement, and
  Documentation labels. Reuse them rather than creating duplicate conventions.
- Bootstrap branch: `chore/TER-5-bootstrap-ratpack`, based on documentation commit
  `00697b09e4ae0b885f31ab7c9fc610d79a5aa566`. Local `devel` points to that
  baseline. As a one-time adoption exception, the bootstrap PR targets `main` so
  the migration can receive independent review before changing the remote branch
  structure. This bootstrap merge is not a product release.
- After accepted review and a successful bootstrap merge, the integration owner
  creates remote `devel` at the reviewed `main` commit, sets it as default, and
  protects it. Normal subsequent PRs target `devel`. Preserve existing history;
  no released version or product readiness is implied.
- GitHub policy is squash merges for task PRs and merge commits for releases;
  disable rebase merges. Require one approving review and the `check` CI job on
  `main` and `devel`, with direct/force pushes and deletion prohibited. Record
  applied settings and read-back evidence in TER-5. Any unavailable protection
  remains explicit and requires the manual integration checklist.
- The `check` GitHub Actions job validates contributor configuration, profiles,
  local Markdown targets, and whitespace. Local checks and the installed
  `rp-bootstrap` doctor must pass before publishing. A successful remote run
  must be observed on the linked PR; local results alone do not establish CI.
- The user connected GitHub and Linear. That establishes an integration, but its
  draft/ready/merged status transitions have not yet been verified. Use manual
  evidence-backed tracker updates until automation targeting `devel` is proven
  against a real linked PR. A ready bootstrap PR alone cannot verify merge or
  `devel` automation.
- All five `.codex/agents/` profile files parse, and the reloaded host advertises
  their native role names. This verifies discovery; successful spawning remains
  untested. No workers have been spawned and no global trust, model, permissions,
  or account settings were changed.

Next handoff: a different human or authorized reviewer reviews the exact
bootstrap PR revision. The integration owner verifies review, GitHub approval,
and CI before merging and completing the remote `devel` migration. The task
stays In Review until accepted merge; product work is outside this handoff.
