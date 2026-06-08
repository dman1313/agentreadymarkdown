# 23 — Content Ownership, DRM, and Legal Notice

**Status:** V1 user-facing requirement  
**Applies to:** All uploads, especially ebooks (EPUB, PDF, MOBI, AZW3, AZW — DRM-free only)

---

## Short notice (UI — upload page)

Use this block on the web upload screen and anywhere users select files:

```text
Content you upload must be yours to use — files you own, created, or are explicitly authorized to convert for your personal or organizational use.

Ebooks must be DRM-free. Do not upload Kindle, Adobe, or other DRM-protected files. AgentReady cannot remove copy protection and will reject encrypted books.

AgentReady is a conversion tool only. HumanGoodAI and AgentReady do not review your files, do not verify ownership, and are not responsible for how you obtain, use, or distribute converted content. You are solely responsible for complying with copyright and licensing laws in your jurisdiction.
```

---

## Checkbox acknowledgement (optional future UI)

If the product adds an explicit acknowledgement step:

```text
☐ I confirm that I own or have the right to convert these files, that any ebooks are DRM-free, and that I accept the AgentReady Terms of Use.
```

V1 may show the notice without a checkbox; the notice still applies.

---

## Export package notice

Every AgentReady export `README.md` must include the **Export legal notice** block below so downstream agent pipelines inherit the disclaimer.

---

## Export legal notice (paste into export README)

```markdown
## Legal notice

**Your content.** This export was created from files you supplied. You represent and warrant that you own each file or have all rights and permissions needed to convert and use it, including for AI agent workflows.

**DRM-free only.** AgentReady supports DRM-free documents only. It does not decrypt, strip, or bypass digital rights management, Kindle DRM, Adobe ACS, or similar copy protection. Encrypted or DRM-protected files are rejected.

**No responsibility.** AgentReady and HumanGoodAI provide this software and export format **"as is"** without warranty of any kind. We do not monitor, verify, or take responsibility for the legality, accuracy, or appropriateness of your source files or how you use converted Markdown. **You are solely responsible** for your uploads, exports, and compliance with applicable copyright, contract, and privacy laws.

**Not legal advice.** This notice is not legal advice. If you are unsure whether you may convert a file, consult qualified counsel or the rights holder before uploading.
```

---

## Converter behavior (DRM)

| Situation | Behavior |
|-----------|----------|
| DRM-free EPUB / MOBI / AZW3 / PDF | Convert when technically possible |
| Password-protected PDF | Reject — `PasswordProtected` |
| Encrypted / DRM ebook | Reject — `PasswordProtected` with DRM-specific user message |
| User attempts to circumvent DRM | Out of scope — never implement |

**User message for DRM rejection:**

```text
This file appears to be encrypted or DRM-protected. AgentReady only supports DRM-free files that you own or are authorized to convert. It cannot remove copy protection.
```

---

## Autobuild / agent instruction

When implementing ebook support, agents must:

1. Surface the **Short notice** on the upload UI.
2. Include the **Export legal notice** in `export.rs` generated `README.md`.
3. Reject DRM/encrypted files — never add decryption.
4. Reference this doc from `docs/PROMPT-AUTOBUILD.md` and `docs/13_INTERFACE_COPY.md`.

---

## Links

- `docs/15_SECURITY_AND_PRIVACY.md`
- `docs/13_INTERFACE_COPY.md`
- `docs/PROMPT-AUTOBUILD.md`
