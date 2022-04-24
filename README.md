## Game of Life

### Another one?

I wanted to use this project to achieve a certain set of goals:

1. Learn more about graphics programming. I could've chosen [Bevy](https://github.com/bevyengine/bevy) which already has a Game of Life plugin, but then I wouldn't learn as much as I would doing it from scratch with [wgpu](https://github.com/gfx-rs/wgpu)
2. Use this as a witness to my knowledge. I can say "I know Rust" or "I know graphics programming" all I want, but nothing will really say how good or bad I really am at it better than my own work

### General Goals

- To make it FAST yet not dealing with headaches of debugging memory issues
- To make it run lean on memory. I want to at least get close to the point where all the memory I am using is all the memory the program would absolutely need
- To make it fancy. The objective is to have it be very customizable and easy to use
- All in all, I want to be able to have absolutely enormous grids in any computer that does not necessarily have the latest hardware available. I do not know if it will be possible even with the most optimized Game of Life, but I at least want tools that give me the peace of mind that it's really not possible and that there isn't something wrong with the tool if in the end it is not a possibility to achieve that

Hence Rust and [wgpu](https://github.com/gfx-rs/wgpu)

### Wait, but this is not complete...

You are seeing this despite it is not completed because I want you to see what I can do despite it is in progress.
Why am I showcasing it if it is still in progress? Because I really want to showcase it, but I don't see myself finishing it soon because

- This takes time and that is something that I would really appreciate right now. With 3 AP exams incoming and me wanting to get 5s in all of them, with classes in which I find it is unacceptable to not do my best in, with other personal things going on, the time that I have to work on this is less than what I would need to finish it within the window I would desirably want to finish it in

### Working On

- Getting the squares to draw nicely with the corner radius
