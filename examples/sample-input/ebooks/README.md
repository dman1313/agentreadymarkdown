# Ebook test samples

Do **not** commit copyrighted ebooks to this repository.

## MOBI / EPUB manual testing

1. Use **DRM-free** files you own (Calibre “DeDRM” is out of scope — buy DRM-free or strip DRM outside AgentReady).
2. Copy a sample to Desktop and upload via `./scripts/serve.sh` or:

   ```bash
   cargo run -- convert /path/to/your-book.mobi --output /tmp/ar-mobi-test
   ```

3. Check `/tmp/ar-mobi-test/documents/` for Markdown and `README.md` legal notice.

## Generating a minimal DRM-free MOBI (optional)

With [Calibre](https://calibre-ebook.com/) installed:

```bash
ebook-convert your-book.epub your-book.mobi
```

Use only on content you have rights to convert.
