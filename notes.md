 My PC is little endian, the CHIP-8 architecture is big endian, that doesn't matter here?

 The stack pointer points to the topmost level of the stack
 meaning that it points to empty space. If stack[0], stack[1], stack[2] have values inside of
 them, then stack_pointer is 3 (it points to stack[3])

 For now I don't check for overflow in certain places, If the necessity occurs, I will
 do it

 The last register V[0xF] seems to be for signifying overflow

 The Timing needs to be configurable, since different games want to run at different
 speeds, but for the first version, 500Hz - 700Hz will do

 The PC should be incremented at the end of every fetch cycle, 
 don't forget that!

 It would be wise to extract
 X, Y, N, NN, NNN before the decoding

 I don't make tests anymore, but at least make the ones I already wrote compile
 with `cargo test` 
