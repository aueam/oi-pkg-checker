# oi-pkg-checker

This application allows you to analyze packages and components from OpenIndiana.
As output, you will get problems found in the analysis, which you can fix.
You can visualize packages and their dependencies with [oi-pkg-visualizer](https://github.com/aueam/oi-pkg-visualizer).

## How to

### Build

Just run `make`, it will download some assets and compiles application.

### Use

- Clone [oi-userland repo](https://github.com/OpenIndiana/oi-userland.git) into `/tmp`
- Run the analysis with `./oi-pkg-checker data run --catalog $(pwd)/assets/catalog.dependency.C --encumbered-catalog $(pwd)/assets/catalog.encumbered.dependency.C --components-path /tmp/oi-userland/components`
    - Output is `data.bin` and `problems.bin`
    - Re-print problems with `./oi-pkg-checker print-problems`
- Update assets with `./oi-pkg-checker data update-assets <args>`

#### Check fmri

You can check fmri with `./oi-pkg-checker SUNWIPython /tmp/oi-userland/components>` to see what packages need
that fmri.

