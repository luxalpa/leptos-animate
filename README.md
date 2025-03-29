# Animation components and tools for [Leptos](https://leptos.dev/)

This crate provides various animation utilities in order to handle different scenarios in your web app:

| Component        | Purpose                                                                                                                                                                |
|------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `AnimatedFor`    | The base animation primitive. It is an equivalent to leptos' `<For />` component and handles lists of elements. Provides FLIP animations for moving elements around.   |
| `AnimatedShow`   | Animate the showing and hiding of a single element.                                                                                                                    |
| `AnimatedSwap`   | Swap out one element with another.                                                                                                                                     |
| `AnimatedLayout` | Like `AnimatedFor`, except it allows to change the container's CSS layout between different configurations (for example moving between different grid configurations). |
| `SizeTransition` | React to size changes on the element and animate between those.                                                                                                        |

https://github.com/luxalpa/leptos-animate/assets/4991312/7ad67edb-95cd-464b-a19e-490fb2668f5c

https://github.com/luxalpa/leptos-animate/assets/4991312/07b14554-2342-444d-92f4-4125babe976f

https://github.com/luxalpa/leptos-animate/assets/4991312/640a2ab4-4b3f-4984-81bd-8ded08426b36

https://github.com/luxalpa/leptos-animate/assets/4991312/bad42f7c-96d9-450e-bd1d-ed848a51a5b2

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

## Todos:

- [ ] Animation Staggering / Delay
- [ ] Support `animated_size` for dynamically sized content using placeholder elements
- [ ] Handle resizes on the container using a `ResizeObserver`.
- [ ] Support View Transitions