# oi-pkg-checker

This application allows you to analyze packages and components from OpenIndiana.
As output, you will get problems found in the analysis, which you can fix.
You can visualize packages and their dependencies with [oi-pkg-visualizer](https://github.com/aueam/oi-pkg-visualizer).

## How to

### Build

Just run `make`, it will download some assets and compiles application.

### Use

- Clone [oi-userland repo](https://github.com/OpenIndiana/oi-userland.git) into `/tmp`
- Run the analysis with `target/release/oi-pkg-checker data run --catalog $(pwd)/assets/catalog.dependency.C --catalog $(pwd)/assets/catalog.encumbered.dependency.C --repo-path /tmp/oi-userland`
    - Output is `data.bin` and `problems.bin`
    - Re-print problems with `target/release/oi-pkg-checker print-problems`
- Update assets with `target/release/oi-pkg-checker data update-assets --catalog $(pwd)/assets/catalog.dependency.C --encumbered-catalog $(pwd)/assets/catalog.encumbered.dependency.C --repo-path /tmp/oi-userland/`

#### Check fmri

You can check fmri with `target/release/oi-pkg-checker check-fmri metapackages/build-essential --repo-path /tmp/oi-userland` to see what packages need
that fmri.
