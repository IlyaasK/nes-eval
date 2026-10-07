# APU Frame Counter

> Source: <https://www.nesdev.org/wiki/APU_Frame_Counter> — NESdev Wiki, revision [23448](https://www.nesdev.org/w/index.php?oldid=23448) (last edited 2026-01-19T03:49:39Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=APU_Frame_Counter&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The **[NES APU](APU.md) frame counter** (or **frame sequencer**) generates low-frequency clocks for the channels and an optional 60 Hz interrupt.
The name "frame counter" might be slightly misleading because the clocks have nothing to do with the video signal.

The frame counter contains the following: [divider](APU.md#Glossary), looping clock [sequencer](APU.md#Glossary), frame interrupt flag.

The sequencer is clocked on every other CPU cycle, so 2 CPU cycles = 1 APU cycle. The sequencer keeps track of how many APU cycles have elapsed in total, and each step of the sequence will occur once that total has reached the indicated amount (with an additional delay of one CPU cycle for the quarter and half frame signals). Once the last step has executed, the count resets to 0 on the next APU cycle.

<table>
<tr><td>Address</td><td>Bitfield</td><td>Description</td></tr>
<tr><td><b>$4017</b></td><td><tt>MI--.----</tt></td><td><b>Set mode and interrupt</b> (write)</td></tr>
<tr><td>Bit 7</td><td><tt>M--- ----</tt></td><td>Sequencer mode: 0 selects 4-step sequence, 1 selects 5-step sequence</td></tr>
<tr><td>Bit 6</td><td><tt>-I-- ----</tt></td><td>Interrupt inhibit flag. If set, the frame interrupt flag is cleared, otherwise it is unaffected.</td></tr>
<tr><td colspan="2">Side effects</td><td>After 3 or 4 CPU clock cycles*, the timer is reset.<br/>If the mode flag is set, then both "quarter frame" and "half frame" signals are also generated.</td></tr></table>

* If the write occurs *during* an APU cycle, the effects occur 3 CPU cycles after the **$4017** write cycle, and if the write occurs *between* APU cycles, the effects occurs 4 CPU cycles after the write cycle.

PAL behavior is currently assumed to be the same.

The frame interrupt flag is connected to the [CPU](CPU.md)'s [IRQ](IRQ.md) line. It is set at a particular point in the 4-step sequence (see below) provided the interrupt inhibit flag in $4017 is clear, and can be cleared either by reading $4015 (which also returns its old status) or by setting the interrupt inhibit flag.

### Mode 0: 4-Step Sequence (bit 7 of $4017 clear)

<table>
<tr><th rowspan="2">Step</th><th colspan="2">APU cycles</th><th rowspan="2"><a href="APU_Envelope.md">Envelopes</a> &amp; <a href="APU_Triangle.md">triangle's linear counter</a><br/>(Quarter frame)</th><th rowspan="2"><a href="APU_Length_Counter.md">Length counters</a> &amp; <a href="APU_Sweep.md">sweep units</a><br/>(Half frame)</th><th rowspan="2">Frame interrupt flag</th></tr>
<tr><th>NTSC</th><th>PAL</th></tr>
<tr><td>1</td><td>3728, PUT</td><td>4156, PUT</td><td>Clock</td><td> </td><td></td></tr>
<tr><td>2</td><td>7456, PUT</td><td>8313, PUT</td><td>Clock</td><td>Clock</td><td></td></tr>
<tr><td>3</td><td>11185, PUT</td><td>12469, PUT</td><td>Clock</td><td> </td><td></td></tr>
<tr><td rowspan="3">4</td><td>14914, GET</td><td>16626, GET</td><td> </td><td> </td><td>Set if interrupt inhibit is clear</td></tr>
<tr><td>14914, PUT</td><td>16626, PUT</td><td>Clock</td><td>Clock</td><td>Set if interrupt inhibit is clear</td></tr>
<tr><td>0 (14915), GET</td><td>0 (16627), GET</td><td> </td><td> </td><td>Set if interrupt inhibit is clear</td></tr>
<tr><td></td><td colspan="2"> </td><td>NTSC: <i>240 Hz (approx.)</i><br/>PAL: <i>200 Hz (approx)</i></td><td>NTSC: <i>120 Hz (approx.)</i><br/>PAL: <i>100 Hz (approx)</i></td><td>NTSC: <i>60 Hz (approx.)</i><br/>PAL: <i>50 Hz (approx)</i></td></tr></table>

In this mode, the interrupt flag is set every 29830 CPU cycles, which is slightly (0.166%) slower than the 29780.5 CPU cycles per NTSC PPU frame.

On the PAL RP2A07, this is every 33254 CPU cycles, still slower than the 33247.5 CPU cycles per PAL PPU frame but much closer (only off by 0.0196%).

This IRQ allows the CPU to keep time in scenarios where no [NMI](NMI.md) signal is provided.

### Mode 1: 5-Step Sequence (bit 7 of $4017 set)

<table>
<tr><th rowspan="2">Step</th><th colspan="2">APU cycles</th><th rowspan="2"><a href="APU_Envelope.md">Envelopes</a> &amp; <a href="APU_Triangle.md">triangle's linear counter</a><br/>(Quarter frame)</th><th rowspan="2"><a href="APU_Length_Counter.md">Length counters</a> &amp; <a href="APU_Sweep.md">sweep units</a><br/>(Half frame)</th></tr>
<tr><th>NTSC</th><th>PAL</th></tr>
<tr><td>1</td><td>3728, PUT</td><td>4156, PUT</td><td>Clock</td><td></td></tr>
<tr><td>2</td><td>7456, PUT</td><td>8313, PUT</td><td>Clock</td><td>Clock</td></tr>
<tr><td>3</td><td>11185, PUT</td><td>12469, PUT</td><td>Clock</td><td></td></tr>
<tr><td>4</td><td>14914, PUT</td><td>16626, PUT</td><td> </td><td></td></tr>
<tr><td rowspan="2">5</td><td>18640, PUT</td><td>20782, PUT</td><td>Clock</td><td>Clock</td></tr>
<tr><td>0 (18641), GET</td><td>0 (20783), GET</td><td> </td><td></td></tr>
<tr><td></td><td colspan="2"> </td><td>NTSC: <i>192 Hz (approx.), uneven timing</i><br/>PAL: <i>160 Hz (approx.), uneven timing</i></td><td>NTSC: <i>96 Hz (approx.), uneven timing</i><br/>PAL: <i>80 Hz (approx.), uneven timing</i></td></tr></table>

In this mode, the frame interrupt flag is never set.

This mode seems to be intended for use with a NMI handler that writes to $4017 once each PPU frame and resets the sequence before it ever reaches step 5, thereby synchronizing the various APU counters to the PPU frame rate and yielding exactly four quarter-frame clocks and two half-frame clocks per PPU frame. This mode has been observed to be primarily used by first-party titles, including the [FDS BIOS](https://www.nesdev.org/wiki/FDS_BIOS#Power-on_and_reset), and these titles do use it in the way described.

*[Punch-Out!!](https://en.wikipedia.org/wiki/Punch-Out!!_(arcade_game))* and *[Donkey Kong 3](https://en.wikipedia.org/wiki/Donkey_Kong_3)* are arcade boards not directly based on the NES, which use the 2A03 CPU as a sound processor in this mode.

<table>
<tr><th colspan="2"><div><a href="https://www.nesdev.org/wiki/Template:APU_nav">V</a> | T</div> <a href="APU.md">APU</a></th></tr>
<tr><td><b>Concepts</b></td><td><a href="APU_registers.md">Registers</a> | <a href="APU_basics.md">Basics</a> | <a href="APU_period_table.md">Period table</a></td></tr>
<tr><td><b>Channels</b></td><td><a href="APU_Pulse.md">Pulse 1 and Pulse 2</a> | <a href="APU_Triangle.md">Triangle</a> | <a href="APU_Noise.md">Noise</a> | <a href="APU_DMC.md">DMC</a></td></tr>
<tr><td><b>Components</b></td><td><a href="APU_Envelope.md">Envelope</a> | <a href="APU_Sweep.md">Sweep</a> | <a>Frame Counter</a> | <a href="APU_Length_Counter.md">Length Counter</a> | <a href="DMA.md">DMA</a> | <a href="APU_Mixer.md">Mixer</a></td></tr>
<tr><td><b>Expansion audio</b></td><td><a href="https://www.nesdev.org/wiki/FDS_audio">2C33 (Disk System)</a> | <a href="https://www.nesdev.org/wiki/MMC5_audio">MMC5</a> | <a href="https://www.nesdev.org/wiki/VRC6_audio">VRC6</a> | <a href="https://www.nesdev.org/wiki/VRC7_audio">VRC7</a> | <a href="https://www.nesdev.org/wiki/Namco_163_audio">Namco 163</a> | <a href="https://www.nesdev.org/wiki/Sunsoft_5B_audio">Sunsoft 5B</a> | <a href="https://www.nesdev.org/wiki/INES_Mapper_086#Registers">µPD7756</a></td></tr></table>
