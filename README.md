# audio-simulator

This software simulates the sound waves by solving wave equation on 2d surface.  
You can specify the properties of each cells and it affects how the waves propagate.  

![audio_sim_draw](https://user-images.githubusercontent.com/29710855/104086939-55576280-529f-11eb-8def-b1c8e3e6c0ac.gif)

# How to use

Run

    cargo run --release

Then a window appears. You can interact with sound waves on it.  
Because this software not really optimized, you can not sound on the realtime :crying_cat_face: :crying_cat_face:.  
When you press quit! button. The software will terminates. and `mic.wav` witch is a sound of your waves appears.  

Table bellow describes parameters and role of buttons.

| parameter                 | descrption                                                                         |
| ------------------------- | ---------------------------------------------------------------------------------- |
| normal mode!              | change mode to mic positioning mode.                                               |
| spec mode!                | change mode to property setting mode. ()                                           |
| left click (normal mode)  | move left mic to cursor position.                                                  |
| right click (normal mode) | move right mic to cursor position.                                                 |
| left click (spec mode)    | change property of a cell under cursor.                                            |
| middle click (spec mode)  | toggle surface pushing. (when it's enabled, it's add force to a cell under cursor) |
| scroll wheel (spec mode)  | change drop f                                                                      |
| drop f                    | the magnitude of force added.                                                      |
| propagation ratio         | how force propagates to cells around it.                                           |
| dumping ratio             | how force decays when it propagates.                                               |
| mic l pos                 | position of the left mic                                                           |
| mic r pos                 | position of the right mic                                                          |
| quit!                     | terminates the program.                                                            |

# LISCENSE

This project is licensed under the Mozilla Public License, v. 2.0 - see the [LICENSE](LICENSE) file for details
