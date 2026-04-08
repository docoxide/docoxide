# scripts

Repo maintenance scripts for docoxide.

## Setup

```sh
cd scripts
uv sync
```

## `bump.py`

Bump the version of one or all bindings.

```sh
task bump -- --help
task bump -- rust patch
task bump -- wasm preminor --preid=beta
task bump -- all 0.2.0-alpha.1
```

Follows the `npm version` convention for keywords and `--preid`.

## `tag.py`

Create release tags for docoxide bindings.

```sh
task tag -- --help
task tag -- --dry-run
task tag -- python --push
```
