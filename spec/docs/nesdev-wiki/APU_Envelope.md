# APU Envelope

> Source: <https://www.nesdev.org/wiki/APU_Envelope> — NESdev Wiki, revision [23446](https://www.nesdev.org/w/index.php?oldid=23446) (last edited 2026-01-19T03:49:36Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=APU_Envelope&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

In a synthesizer, an [envelope](https://en.wikipedia.org/wiki/ADSR_envelope) is the way a sound's parameter changes over time.
The NES [APU](APU.md) has an envelope generator that controls the volume in one of two ways: it can generate a decreasing saw envelope (like a decay phase of an [ADSR](http://en.wikipedia.org/wiki/Synthesizer#ADSR_envelope)) with optional looping, or it can generate a constant volume that a more sophisticated software envelope generator can manipulate. Volume values are practically linear (see: [APU Mixer](APU_Mixer.md)).

Each volume envelope unit contains the following: start flag, [divider](APU.md#Glossary), and decay level counter.

```
                                   Loop flag
                                        |
               Start flag  +--------.   |   Constant volume
                           |        |   |        flag
                           v        v   v          |
Quarter frame clock --> Divider --> Decay --> |    |
                           ^        level     |    v
                           |                  | Select --> Envelope output
                           |                  |
        Envelope parameter +----------------> |
```

<table>
<tr><th>Address</th><th>Bitfield</th><th>Description</th></tr>
<tr><td><b>$4000</b></td><td><tt>ddLC.VVVV</tt></td><td><b><a href="APU_Pulse.md">Pulse channel 1</a></b> duty and volume/envelope (write)</td></tr>
<tr><td><b>$4004</b></td><td><tt>ddLC.VVVV</tt></td><td><b><a href="APU_Pulse.md">Pulse channel 2</a></b> duty and volume/envelope (write)</td></tr>
<tr><td><b>$400C</b></td><td><tt>--LC.VVVV</tt></td><td><b><a href="APU_Noise.md">Noise channel</a></b> volume/envelope (write)</td></tr>
<tr><td>bit 5</td><td><tt>--L- ----</tt></td><td><a href="APU_Length_Counter.md">APU Length Counter</a> halt flag/envelope loop flag</td></tr>
<tr><td>bit 4</td><td><tt>---C ----</tt></td><td>Constant volume flag (0: use volume from envelope; 1: use constant volume)</td></tr>
<tr><td>bits 3-0</td><td><tt>---- VVVV</tt></td><td>Used as the volume in constant volume (C set) mode. Also used as the reload value for the envelope's divider (the period becomes V + 1 quarter frames).</td></tr>
<tr><td colspan="3"></td></tr>
<tr><td><b>$4003</b></td><td><tt>LLLL.Lttt</tt></td><td><b><a href="APU_Pulse.md">Pulse channel 1</a></b> <a href="APU_Length_Counter.md">length counter load</a> and <a href="APU.md#Glossary">timer</a> (write)</td></tr>
<tr><td><b>$4007</b></td><td><tt>LLLL.Lttt</tt></td><td><b><a href="APU_Pulse.md">Pulse channel 2</a></b> <a href="APU_Length_Counter.md">length counter load</a> and <a href="APU.md#Glossary">timer</a> (write)</td></tr>
<tr><td><b>$400F</b></td><td><tt>LLLL.L---</tt></td><td><b><a href="APU_Noise.md">Noise channel</a></b> <a href="APU_Length_Counter.md">length counter load</a> (write)</td></tr>
<tr><td colspan="2">Side effects</td><td>Sets start flag</td></tr></table>

When clocked by the [frame counter](APU_Frame_Counter.md), one of two actions occurs: if the start flag is clear, the divider is clocked, otherwise the start flag is cleared, the decay level counter is loaded with 15, and the divider's period is immediately reloaded.

When the divider is clocked while at 0, it is loaded with V and clocks the decay level counter. Then one of two actions occurs: If the counter is non-zero, it is decremented, otherwise if the loop flag is set, the decay level counter is loaded with 15.

The envelope unit's volume output depends on the constant volume flag: if set, the envelope parameter directly sets the volume, otherwise the decay level is the current volume. The constant volume flag has no effect besides selecting the volume source; the decay level will still be updated when constant volume is selected.

Each of the three envelope units' output is fed through additional gates at the [sweep unit](APU_Sweep.md) (pulse only), waveform generator ([sequencer](APU.md#Glossary) or LFSR), and [length counter](APU_Length_Counter.md).

<table>
<tr><th colspan="2"><div><a href="https://www.nesdev.org/wiki/Template:APU_nav">V</a> | T</div> <a href="APU.md">APU</a></th></tr>
<tr><td><b>Concepts</b></td><td><a href="APU_registers.md">Registers</a> | <a href="APU_basics.md">Basics</a> | <a href="APU_period_table.md">Period table</a></td></tr>
<tr><td><b>Channels</b></td><td><a href="APU_Pulse.md">Pulse 1 and Pulse 2</a> | <a href="APU_Triangle.md">Triangle</a> | <a href="APU_Noise.md">Noise</a> | <a href="APU_DMC.md">DMC</a></td></tr>
<tr><td><b>Components</b></td><td><a>Envelope</a> | <a href="APU_Sweep.md">Sweep</a> | <a href="APU_Frame_Counter.md">Frame Counter</a> | <a href="APU_Length_Counter.md">Length Counter</a> | <a href="DMA.md">DMA</a> | <a href="APU_Mixer.md">Mixer</a></td></tr>
<tr><td><b>Expansion audio</b></td><td><a href="https://www.nesdev.org/wiki/FDS_audio">2C33 (Disk System)</a> | <a href="https://www.nesdev.org/wiki/MMC5_audio">MMC5</a> | <a href="https://www.nesdev.org/wiki/VRC6_audio">VRC6</a> | <a href="https://www.nesdev.org/wiki/VRC7_audio">VRC7</a> | <a href="https://www.nesdev.org/wiki/Namco_163_audio">Namco 163</a> | <a href="https://www.nesdev.org/wiki/Sunsoft_5B_audio">Sunsoft 5B</a> | <a href="https://www.nesdev.org/wiki/INES_Mapper_086#Registers">µPD7756</a></td></tr></table>
