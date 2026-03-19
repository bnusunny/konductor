# Questioning Guide for Project Discovery

## Purpose

This guide helps the konductor-discoverer agent extract project vision, requirements, and roadmap through structured questioning.

## Question Design Principles

1. **Ask one question at a time** — Wait for the response before asking the next
2. **Prefer multiple choice when possible** — Makes it easier for users to respond quickly
3. **Keep it short** — 5-8 questions maximum for v1 discovery
4. **Be specific** — Avoid vague questions like "tell me about your project"
5. **Confirm and clarify** — Repeat back what you heard to ensure understanding

## Question Categories

### 1. Vision & Purpose

**Goal:** Understand what problem the project solves and why it exists.

Example questions:
- "What are you building? Describe it in one sentence."
- "What problem does this solve?"
- "Why build this now?"

### 2. Target Users

**Goal:** Identify who will use the project and how.

Example questions:
- "Who will use this? (developers, end users, internal team, customers)"
- "How technical is your target audience?"
- "Will this be public/open-source or private/internal?"

### 3. Tech Stack

**Goal:** Determine technical constraints and preferences.

Example questions:
- "What's your tech stack preference? (or should I suggest one)"
- "Are there any required frameworks or libraries?"
- "What programming language(s)?"
- For brownfield: "I found [tech stack from codebase analysis]. Any changes planned?"

### 4. Key Features

**Goal:** Define the minimum viable product (v1) scope.

Example questions:
- "What are the must-have features for v1?"
- "What's the single most important capability?"
- "Which features can wait for v2?"

### 5. Constraints

**Goal:** Identify hard limits on timeline, budget, integrations, or dependencies.

Example questions:
- "Any hard deadlines? (launch date, demo, customer commitment)"
- "Budget or resource constraints?"
- "Required integrations? (APIs, databases, services)"
- "Any compliance or security requirements?"

### 6. Out of Scope

**Goal:** Explicitly define what is NOT being built to avoid scope creep.

Example questions:
- "What's explicitly NOT in scope for v1?"
- "What features have you decided to skip?"
- "Any common expectations you want to push back on?"

## Output Templates

### project.md

Structure:
```markdown
# {Project Name}

## Vision
{One-paragraph description of what this project is and why it exists}

## Target Users
{Description of who will use this and how}

## Tech Stack
- Language: {primary language}
- Framework: {if applicable}
- Database: {if applicable}
- Key Libraries: {list any required dependencies}

## Constraints
- {List any hard constraints: deadlines, budgets, integrations, compliance}
```

### requirements.md

Structure:
```markdown
# Requirements

## V1 Requirements (Must-Have)

- **REQ-01**: {First requirement}
  - Acceptance: {How to know it's done}

- **REQ-02**: {Second requirement}
  - Acceptance: {How to know it's done}

... continue for all v1 requirements ...

## V2 Requirements (Nice-to-Have)

- {Feature or improvement for future versions}
- {Another future feature}

## Explicitly Out of Scope

- {Feature explicitly not being built}
- {Another excluded feature}
```

### roadmap.md

Structure:
```markdown
# Development Roadmap

## Phase 01: {Phase Name}
**Goal:** {What this phase accomplishes}

**Deliverables:**
- {Deliverable 1}
- {Deliverable 2}

**Success Criteria:**
- {How to know this phase is complete}

## Phase 02: {Phase Name}
**Goal:** {What this phase accomplishes}

**Deliverables:**
- {Deliverable 1}
- {Deliverable 2}

**Success Criteria:**
- {How to know this phase is complete}

... continue for all phases ...

## Estimated Timeline
- Phase 01: {rough estimate if known, or "TBD"}
- Phase 02: {rough estimate if known, or "TBD"}
```

## Question Flow Example

For a new web app project:

1. "What are you building? Describe it in one sentence."
   → User: "A task management app for remote teams"

2. "Who will use this? (a) developers, (b) end users/customers, (c) internal team"
   → User: "b - end users, small remote teams"

3. "What's your tech stack preference? (a) Node.js/React, (b) Python/Django, (c) Rails, (d) suggest one"
   → User: "a - Node.js and React"

4. "What are the 3-5 must-have features for v1?"
   → User: "Create tasks, assign to team members, track status, simple kanban view"

5. "Any hard constraints? (deadlines, required integrations, compliance)"
   → User: "Need to integrate with Slack, launch in 3 months"

6. "What's explicitly NOT in scope for v1?"
   → User: "No mobile app, no time tracking, no advanced reporting"

After gathering responses, synthesize into the three output files.

## Tips for Brownfield Projects

When working with existing codebases:

1. **Start with codebase analysis context** — Reference the structure and tech stack discovered by konductor-codebase-mapper
2. **Validate assumptions** — "I found you're using {tech}. Is that still the plan?"
3. **Ask about gaps** — "What's missing that you need to build?"
4. **Identify refactoring needs** — "Any parts that need to be rewritten or improved?"
5. **Clarify existing vs. new** — "Which requirements are new features vs. improvements to existing code?"

## Common Pitfalls to Avoid

- **Don't ask leading questions** — Let the user define priorities
- **Don't assume technical knowledge** — Explain terms if needed
- **Don't skip out-of-scope** — This is critical for preventing scope creep
- **Don't write vague requirements** — Each requirement should have clear acceptance criteria
- **Don't create too many phases** — 3-6 phases is typical for v1 projects
