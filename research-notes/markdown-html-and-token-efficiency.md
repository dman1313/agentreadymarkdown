# Research Notes: Markdown, HTML, and Token Efficiency

## Summary

AgentReady should explain Markdown as a practical bridge format for people and agents.

Markdown is not magic. Agents can read HTML, PDFs, Word documents, and spreadsheets. The product claim should be that Markdown can reduce formatting noise and make knowledge clearer. It should not promise fixed token or electricity savings.

## Markdown

CommonMark provides a formal Markdown specification and test suite. This supports treating Markdown as a widely understood plain-text markup format rather than a proprietary AgentReady format.

Reference:

https://spec.commonmark.org/

## HTML

HTML is a web markup language. MDN describes HTML as using elements, opening tags, content, closing tags, attributes, nesting, and document structure.

That makes HTML excellent for web pages, but it can include structure and styling information that is not always useful when packaging clean knowledge for an agent.

Reference:

https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Structuring_content/Basic_HTML_syntax

## Token efficiency

OpenAI API documentation discusses prompt size, caching, latency, and input token costs. This supports a cautious product claim that reducing unnecessary input noise can support efficiency.

Reference:

https://developers.openai.com/api/docs/guides/prompt-caching

## Allowed claim

Cleaner Markdown can help reduce unnecessary token use by removing extra formatting and keeping important content clearer.

## Avoided claim

AgentReady must not claim guaranteed token savings, guaranteed electricity savings, or exact environmental impact.
