# Skill: Q&A Brainstorming Protocol

## Purpose

Help Dwayne work through ideas, specs, products, agents, prompts, and decisions one question at a time using a structured brainstorming format.

## Trigger Phrases

Use this skill when Dwayne says:

- “Q&A brainstorming protocol”
- “Q&A1”
- “help me work through this”
- “ask me questions one by one”
- “let’s brainstorm this”
- “help me build the spec”

## Core Rule

Ask only one question at a time.

Never ask a list of questions unless Dwayne directly asks for a list.

## Question Format

Each question must include simple answer options:

```text
A. Option one
B. Option two
C. Option three
D. Other
```

When helpful, include:

```text
My recommendation: B.
```

## Workflow

1. Ask one focused question.
2. Provide A, B, C, and Other options.
3. Wait for Dwayne’s answer.
4. Summarize the answer in clean spec-ready wording.
5. Ask:

```text
Are we done with this question?

A. Yes, move on
B. Add more
C. Revise this answer
D. Other
```

6. Only move to the next question after Dwayne confirms.

## Choice Interface Requirement

When the interface supports it, present options through clickable choices or selectable buttons.

If not supported, render Markdown choices.

## Voice Dictation Support

Dwayne may answer quickly by voice. Interpret messy speech naturally and convert it into clear decisions. Preserve meaning.

## Output Style

Use Markdown. Keep each step short. Avoid asking multiple questions at once.
