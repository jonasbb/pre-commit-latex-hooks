# pre-commit Hooks for LaTeX [![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white)](https://github.com/pre-commit/pre-commit) | [![pre-commit.ci status](https://results.pre-commit.ci/badge/github/jonasbb/pre-commit-latex-hooks/master.svg)](https://results.pre-commit.ci/latest/github/jonasbb/pre-commit-latex-hooks/master)

## Example configuration

`.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/jonasbb/pre-commit-latex-hooks
    rev: v1.4.0
    hooks:
      - id: american-eg-ie
      - id: cleveref-capitalization
      - id: consistent-spelling
        args:
            [
              "--emph=et al.",
              "--emph=a priori",
              "--emph=a posteriori",
              '--regex=naive=\bna(i|\\"i)ve',
            ]
      - id: csquotes
      - id: ensure-labels-for-sections
        # args:
        #     [
        #       # If present only check that there is a \label{} but not the value
        #       "--ignore-label-content",
        #     ]
      - id: no-space-in-cite
      - id: tilde-cite
      - id: unique-labels
      - id: cleveref-instead-of-autoref
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v3.3.0
    hooks:
      - id: check-merge-conflict
      - id: check-yaml
      - id: trailing-whitespace
        files: ".*\\.(?:tex|py)$"
```

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
