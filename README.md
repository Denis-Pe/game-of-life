## Game of Life

### Another one?

I wanted to use this project to achieve a certain set of goals:

1. Learn more about graphics programming. I could've chosen [Bevy](https://github.com/bevyengine/bevy) which already has a Game of Life plugin, but then I wouldn't learn as much as I would doing it from scratch with [wgpu](https://github.com/gfx-rs/wgpu)
2. For fun, as well as other personal reasons

### General Goals

- To make it FAST
- To make it run lean on memory. I want to at least get close to the point where all the memory I am using is all the memory the program would absolutely need
- To make it fancy; the objective is to have it be very customizable and easy to use
- All in all, I want to be able to have absolutely enormous grids in any computer that does not necessarily have the latest hardware available. I do not know if it will be possible even with the most optimized Game of Life, but I at least want tools that give me the peace of mind that it's really not possible or I haven't done as best as I could and that there isn't something wrong with the tool if in the end it is not a possibility to achieve that

Hence [Rust](https://www.rust-lang.org/) and [wgpu](https://github.com/gfx-rs/wgpu) aside from me liking Rust and, again, wanting to learn more about graphics programming.

### To-Do

#### Current

- Be able to turn squares on and off

#### Postponed

- Getting squares to draw with a nice corner radius
  - To get it right in terms of both low-cost and also nice looks will require more time and investigation. This small thing is holding me back from doing more crucial things that require less time and investigation, I want to get to the juicy stuff already!