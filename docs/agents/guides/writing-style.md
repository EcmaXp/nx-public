# Writing Style

Write in the style of the [Google developer documentation style guide](https://developers.google.com/style), tempered by Zinsser's four principles. These rules govern all prose: responses, visible thinking, documents, PR descriptions, and commit message bodies. They apply in Korean as much as in English: they are about structure, not vocabulary.

## Zinsser's four principles (the goal)

1. Simplicity: strip every idea to its cleanest form; prefer the plain word over the impressive one
1. Brevity: every sentence must earn its place; cut what does not change what the reader does next
1. Clarity: if the reader must reread a sentence, the writer failed; leave one possible meaning
1. Humanity: write like a person, not a spec; warmth and directness are allowed

## Google style rules (the method)

- Conversational and friendly tone, without being frivolous: like a knowledgeable colleague, not a spec
- Address the reader as "you"; use "we" only for a real group
- Active voice; name the actor ("the hook rewrites the command", not "the command is rewritten")
- Present tense unless the timing itself matters
- Put the condition before the instruction ("To free the port, stop the service", not "Stop the service to free the port")
- Short sentences for a global audience; no idioms or culture-bound references
- One term per concept: pick one name and reuse it; never rotate synonyms for variety
- Cut filler: no "please", "simply", "easily", "just"
- Spell out Latin abbreviations in prose: "for example", not "e.g."
- Sentence case for headings
- Numbered lists for sequences, bulleted lists for other parallel items; prose for reasoning and causality
- Prefer bullets to tables for explanations and action items. Keep diagrams small: show a short overview and fold detailed flows or secondary-language text
- Descriptive link text: name the destination, never "click here"
- Serial comma

## Thinking

- Think in the same style: short declarative sentences, active voice, conclusion before justification
- These rules shape how visible thinking reads, not how much you reason: reason as deeply as the task needs, then write it plainly
- No filler openers ("Interesting...", "Let me think about this")

## Precedence

When a style rule and humanity conflict, humanity wins: the reader is a person, not a parser. The Typography section overrides Google punctuation guidance: Google style allows em dashes, this document does not.

## Typography

- Avoid the em-dash (`—`); it reads as machine-generated. Use what humans here actually reach for instead:
  - Colon (`:`) to introduce an explanation, definition, or list
  - Middle dot (`·`) to join paired or related terms (for example `참조·순환`, `의도·문제`)
  - Arrow (`→`) for sequence or flow (for example `입력 → 처리 → 출력`)
  - Parentheses for an aside or clarification
  - Or split into two sentences, or join with a comma
- Avoid the en-dash (`–`): plain hyphen for ranges, or `~` in Korean (`2024~2025`)
- Use straight quotes (`"` `'`), not curly quotes (`“” ‘’`)
- Use three dots (`...`), not the ellipsis character (`…`)
