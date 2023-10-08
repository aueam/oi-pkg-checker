# oi-pkg-checker

This application allows you to analyze packages and components from OpenIndiana.
As output, you will get problems found in the analysis, which you can fix.
You can visualize packages and their dependencies with [oi-pkg-visualizer](https://github.com/aueam/oi-pkg-visualizer).

## How to

### Build

Just run `make`, it will download some assets and compiles application.

### Use

- Run the analysis with `target/release/oi-pkg-checker run --catalog $(pwd)/assets/catalog.dependency.C --catalog $(pwd)/assets/catalog.encumbered.dependency.C`
    - Output is `data.bin` and `problems.bin`
    - Re-print problems with `target/release/oi-pkg-checker print-problems`
- Update assets with `make update`

#### Check fmri

You can check fmri with `target/release/oi-pkg-checker check-fmri metapackages/build-essential` to see what packages need
that fmri.
