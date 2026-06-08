# Scripts

## `serve.sh` — Start the local UI (use this)

```bash
./scripts/serve.sh
./scripts/serve.sh --port 3001
./scripts/serve.sh --no-open
```

Builds into `./target`, picks a free port if busy, opens the browser, and keeps the server running until Ctrl+C.

## `gh-auth.sh` — GitHub login helper

Fixes invalid/expired `gh` tokens (e.g. keyring errors) without putting secrets in the repo.

```bash
chmod +x scripts/gh-auth.sh   # once
./scripts/gh-auth.sh
```

What it does:

1. Shows git remote and branch
2. Runs `gh auth status`
3. If needed, prompts then runs `gh auth login` (browser or token **in terminal only**)
4. Verifies access to `origin` on GitHub

**Never** paste tokens into chat, commit them, or save them under this project.
