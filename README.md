# oi-pkg-checker

This application allows you to analyze packages and components from OpenIndiana.
As output, you will get problems found in the analysis, which you can fix.
You can visualize packages and their dependencies with [oi-pkg-visualizer](https://github.com/aueam/oi-pkg-visualizer).

## How to

### Build

Just run `make`. It will download some assets and compile application.

### Update oi-pkg-checker

1. update repo with `git pull`
2. update assets with `make update`
3. re-compile app with `make build_release`

### Use

- Run the analysis
  with
  `target/release/oi-pkg-checker run --catalog $(pwd)/assets/catalog.dependency.C --catalog $(pwd)/assets/catalog.encumbered.dependency.C`
    - Output is `data.bin`
- Print problems with `target/release/oi-pkg-checker print-problems`

#### Check fmri

You can check fmri with `target/release/oi-pkg-checker check-fmri metapackages/build-essential` to see what packages
need that fmri and other details.
