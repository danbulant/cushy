# Gooey

![Gooey is considered experimental and unsupported](https://img.shields.io/badge/status-prototype-blueviolet)
[![crate version](https://img.shields.io/crates/v/gooey.svg)](https://crates.io/crates/gooey)
[![Documentation for `main` branch](https://img.shields.io/badge/docs-main-informational)](https://gooey.rs/main/gooey/)

Gooey is an experimental Graphical User Interface (GUI) crate for the Rust
programming language. It is built using [`Kludgine`][kludgine], which is powered
by [`winit`][winit] and [`wgpu`][wgpu]. It is incredibly early in development,
and is being developed for a game that will hopefully be developed shortly.

The [`Widget`][widget] trait is the building block of Gooey: Every user
interface element implements `Widget`. A full list of built-in widgets can be
found in the [`gooey::widgets`][widgets] module.

Gooey uses a reactive data model. To see [an example][button-example] of how
reactive data models work, consider this example that displays a button that
increments its own label:

```rust,ignore
// Create a dynamic usize.
let count = Dynamic::new(0_usize);

// Create a new button with a label that is produced by mapping the contents
// of `count`.
Button::new(count.map_each(ToString::to_string))
    // Set the `on_click` callback to a closure that increments the counter.
    .on_click(count.with_clone(|count| move |_| count.set(count.get() + 1)))
    // Run the button as an an application.
    .run()
```

[widget]: https://gooey.rs/main/gooey/widget/trait.Widget.html
[kludgine]: https://github.com/khonsulabs/kludgine
[wgpu]: https://github.com/gfx-rs/wgpu
[winit]: https://github.com/rust-windowing/winit
[widgets]: https://gooey.rs/main/gooey/widgets/index.html
[button-example]: https://github.com/khonsulabs/gooey/tree/main/examples/button.rs

## Open-source Licenses

This project, like all projects from [Khonsu Labs](https://khonsulabs.com/), is open-source.
This repository is available under the [MIT License](./LICENSE-MIT) or the
[Apache License 2.0](./LICENSE-APACHE).

To learn more about contributing, please see [CONTRIBUTING.md](./CONTRIBUTING.md).
