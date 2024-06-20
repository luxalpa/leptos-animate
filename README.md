# Animation components and tools for [Leptos](https://leptos.dev/)

This crate provides various animation utilities in order to handle different scenarios in your web app:

| Component        | Purpose                                                                                                                                                               |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AnimatedFor`    | The base animation primitive. It is an equivalent to leptos' `<For />` component and handles lists of elements. Provides FLIP animations for moving elements around.  |
| `AnimatedShow`   | Animate the showing and hiding of a single element                                                                                                                    |
| `AnimatedSwap`   | Swap out one element with another                                                                                                                                     |
| `AnimatedLayout` | Like `AnimatedFor`, except it allows to change the container's CSS layout between different configurations (for example moving between different grid configurations) |
| `SizeTransition` | React to size changes on the element and animate between those.                                                                                                       |

## Usage

See the project in the `examples` subdirectory.

## How it works

Most of the components use `AnimatedFor` under the hood. Whenever the input to that component
changes, we check which elements got added and removed and run the according enter/leave animations.
We also take a snapshot of all previous components positions (and sizes) and then compare it to
their new positions and then animate between them.

For leave animations, we set them to `position:absolute` in order for them to not take up any more
space in the layout. Because their size often depends on the parent container (like with elements
that have `width:100%` or are part of a grid), we must lock-in their size while they animate out.
We also dispose their reactive scope, so during the leave animations there will be no more reactive
changes.

For enter animations, we wait until the child view has been rendered using a `queue_microtask` tick,
then we extract the reference to the DOM node from the view to use for all further animations.

Move animations are being done using the CSS `transform` property.

## Todo's:

- [ ] Animation Staggering / Delay
- [ ] Support `animated_size` for dynamicly sized contents using placeholder elements
- [ ] Handle resizes on the container using a `ResizeObserver`.
