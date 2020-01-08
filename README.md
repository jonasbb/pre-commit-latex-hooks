# pre-commit Hooks for LaTeX

## Example configuration

`.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/jonasbb/pre-commit-latex-hooks
    rev: v1.1.0
    hooks:
      - id: american-eg-ie
      - id: cleveref-capitalization
      - id: csquotes
      - id: no-space-in-cite
      - id: tilde-cite
      - id: consistent-spelling
        args:
            [
              "--emph=et al.",
              "--emph=a priori",
              "--emph=a posteriori",
              '--regex=naive=\bna(i|\\"i)ve',
            ]
      - id: ensure-labels-for-sections
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v2.2.3
    hooks:
      - id: check-merge-conflict
      - id: check-yaml
      - id: trailing-whitespace
        files: ".*\\.(?:tex|py)$"
```
