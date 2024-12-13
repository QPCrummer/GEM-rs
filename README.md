# GEM.rs
## Greenhouse Environment Monitor
A greenhouse monitoring solution written in rust.

<!-- TABLE OF CONTENTS -->
<details open="open">
  
  <summary><h2 style="display: inline-block">Table of Contents</h2></summary>
  <ol>
    <li><a href="#markdown-header-requirements">Requirements</a></li>
    <li><a href="#installation-of-development-dependencies">Installation of development dependencies</a></li>
    <li><a href="#running">Running</a></li>
    <li><a href="#features">Features</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</details>

<!-- Requirements -->
<details open="open">
  <summary><h2 style="display: inline-block" id="requirements">Requirements</h2></summary>
  
- The standard Rust tooling (cargo, rustup) which you can install from https://rustup.rs/

- Toolchain support for the cortex-m0+ processors in the rp2040 (thumbv6m-none-eabi)

- flip-link - this allows you to detect stack-overflows on the first core, which is the only supported target for now.

- An embedded system with at least 120KB of memory

</details>

<!-- Installation of development dependencies -->
<details open="open">
  <summary><h2 style="display: inline-block" id="installation-of-development-dependencies">Installation of development dependencies</h2></summary>

```sh
rustup target install thumbv6m-none-eabi
cargo install flip-link
# Installs the elf2uf2-rs runner
cargo install --locked elf2uf2-rs
```
</details>


<!-- Running -->
<details open="open">
  <summary><h2 style="display: inline-block" id="running">Running</h2></summary>
  
For a debug build
```sh
cargo run
```
For a release build
```sh
cargo run --release
```
</details>

<!-- Features -->
<details open="open">
  <summary><h2 style="display: inline-block" id="features">Features</h2></summary>
  The following features are part of the current release of GEM-rs:

- Temperature monitoring and safety range
- Humidity monitoring and safety range
- Pressure monitoring
- Uptime tracker
- Watering system scheduler
- Smoke/fire detection support
</details>

<!-- ROADMAP -->

## Roadmap

This project is unlikely to be developed further as it was only a proof of concept for a demonstration.

See the [open issues](https://github.com/QPCrummer/GEM-rs/issues) for a list of
proposed features (and known issues).

## License

The contents of this repository are licensed under the _MIT License. 
See `MIT` for more information on each specific licence.

Any submissions to this project (e.g. as Pull Requests) must be made available
under these terms.

## Contact

Raise an issue: [GitHub Issues](https://github.com/QPCrummer/GEM-rs/issues)
