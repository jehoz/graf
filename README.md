# GRAF

Graf is an experimental music sequencer where you create dynamic musical patterns using boolean circuits

[![demo video](https://img.youtube.com/vi/QiUZdrTPDJc/0.jpg)](https://www.youtube.com/watch?v=QiUZdrTPDJc)


# Building and running

Graf is still in development but usable enough to play around with if you want.

You'll need to have the [rust toolchain](https://rust-lang.org/tools/install/) installed, then clone or download this repo somewhere on your computer.  
Then all you need to do is run `cargo run` from inside the graf folder.  It might take a minute

# How do I use this thing?

Graf outputs MIDI notes (not audio) so you will need a DAW or standalone synth to start making music with it.
You can connect to a MIDI port with the "MIDI Setup" menu in the top left of the window.

Right click somewhere in the background to create a "device".
Then you can right-click and drag from one device to another to connect them with a "wire".  If you hold Shift while creating a wire, you will create a negative wire which inverts the output of the device the wire is coming from.

Left-clicking on any device will open an inspector window in the top right which exposes the parameters for that device.
You can click and drag devices to move them around, or middle-click and drag to pan the entire viewport.

At the bottom of the window you can pause and play your circuit, and change the BPM.

## Devices

Devices are what drive your musical sequences.  Each device is very simple on its own, but when they're wired together, they can create cool dynamic patterns.

### Clock
Clocks don't take any inputs and output a signal that alternates between ON and OFF at some regular frequency.  They can be synced to the BPM or run independently.
You can also modify the phase offset and the proportion of time spent ON and OFF.

### Gate
Gates take any number of inputs and combine them into one output signal using one of the following boolean logic operations: AND, OR, XOR, NAND, NOR, XNOR.

### Trigger
A trigger takes a single input, and when that signal goes from OFF to ON, the trigger outputs ON for a set amount of time, and outputs OFF the rest of the time.

### Latch
A latch has an internal state (ON or OFF) which is also its output.  The latch takes a single input which will flip the value of its internal state whenever the input signal goes from OFF to ON.

### Note
A note is what turns an ON/OFF signal into an actual MIDI event.  Notes take a single input and send messages out of the connected MIDI port to turn a note on or off.
