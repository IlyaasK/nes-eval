# APU Length Counter

> Source: <https://www.nesdev.org/wiki/APU_Length_Counter> — NESdev Wiki, revision [24331](https://www.nesdev.org/w/index.php?oldid=24331) (last edited 2026-09-30T00:46:59Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=APU_Length_Counter&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The length counter provides automatic duration control for the [NES APU](APU.md) waveform channels. Once loaded with a value, it can optionally count down (when the length counter halt flag is clear). Once it reaches zero, the corresponding channel is silenced.

<table>
<tr><th>Address</th><th>Bitfield</th><th>Description</th></tr>
<tr><td><b>$4015</b></td><td><tt>---d.nt21</tt></td><td><b><a href="APU_DMC.md">DMC</a> control and length counter <a href="APU.md#Status_($4015)">enabled flags</a></b> (write)</td></tr>
<tr><td colspan="3"></td></tr>
<tr><td><b>$4000</b></td><td><tt>ssHc.vvvv</tt></td><td><b><a href="APU_Pulse.md">Pulse channel 1</a> duty cycle, length counter halt, constant volume flag, and volume/<a href="APU_Envelope.md">envelope</a></b> (write)</td></tr>
<tr><td><b>$4004</b></td><td><tt>ssHc.vvvv</tt></td><td><b><a href="APU_Pulse.md">Pulse channel 2</a> duty cycle, length counter halt, constant volume flag, and volume/<a href="APU_Envelope.md">envelope</a></b> (write)</td></tr>
<tr><td><b>$400C</b></td><td><tt>--Hc.vvvv</tt></td><td><b><a href="APU_Noise.md">Noise channel</a> length counter halt, constant volume flag, and volume/<a href="APU_Envelope.md">envelope</a></b> (write)</td></tr>
<tr><td>bit 5</td><td><tt>--H- ----</tt></td><td>Halt length counter (this bit is also the <a href="APU_Envelope.md">envelope's loop flag</a>)</td></tr>
<tr><td colspan="3"></td></tr>
<tr><td><b>$4008</b></td><td><tt>Hlll.llll</tt></td><td><b><a href="APU_Triangle.md">Triangle channel</a> length counter halt and linear counter load</b> (write)</td></tr>
<tr><td>bit 7</td><td><tt>H--- ----</tt></td><td>Halt length counter (this bit is also the <a href="APU_Triangle.md">linear counter's control flag</a>)</td></tr>
<tr><td colspan="3"></td></tr>
<tr><td><b>$4003</b></td><td><tt>LLLL.Lttt</tt></td><td><b><a href="APU_Pulse.md">Pulse channel 1</a> length counter load and <a href="APU_Pulse.md">timer</a></b> (write)</td></tr>
<tr><td><b>$4007</b></td><td><tt>LLLL.Lttt</tt></td><td><b><a href="APU_Pulse.md">Pulse channel 2</a> length counter load and <a href="APU_Pulse.md">timer</a></b> (write)</td></tr>
<tr><td><b>$400B</b></td><td><tt>LLLL.Lttt</tt></td><td><b><a href="APU_Triangle.md">Triangle channel</a> length counter load and <a href="APU_Triangle.md">timer</a></b> (write)</td></tr>
<tr><td><b>$400F</b></td><td><tt>LLLL.L---</tt></td><td><b><a href="APU_Noise.md">Noise channel</a> length counter load</b> (write)</td></tr>
<tr><td>bits 7-3</td><td><tt>LLLL L---</tt></td><td>If the <a href="APU.md#Status_($4015)">enabled flag</a> is set, the length counter is loaded with entry L of the length table:
<pre>     |  0   1   2   3   4   5   6   7    8   9   A   B   C   D   E   F
-----+----------------------------------------------------------------
00-0F  10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
10-1F  12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
</pre></td></tr>
<tr><td colspan="2">Side effects</td><td>The <a href="APU_Envelope.md">envelope is restarted</a>, for pulse channels phase is reset, for triangle the linear counter reload flag is set.</td></tr></table>

## Clocking

When the [enabled](APU.md#Status_($4015)) bit is cleared (via **$4015**), the length counter is forced to 0 and cannot be changed until enabled is set again (the length counter's previous value is lost). There is no immediate effect when enabled is set.

When clocked by the [frame counter](APU_Frame_Counter.md), the length counter is decremented *except* when:

* The length counter is 0, or
* The halt flag is set

## Length counter internals

In the actual APU, the length counter silences the channel when clocked *while already zero* (provided the length counter halt flag isn't set). The values in the tables presented here are the actual values the length counter gets loaded with *plus one*, to allow us to use a model where the channel is silenced when the length counter *becomes* zero.

The [triangle's linear counter](APU_Triangle.md#Linear_counter_control_($4008,_write)) works differently, and does halt the channel when it *reaches* zero.

## Length table

The below table maps the length counter index to their corresponding length value.

| Length counter index | Actual length |
|---|---|
| $00 | 10 |
| $01 | 254 |
| $02 | 20 |
| $03 | 2 |
| $04 | 40 |
| $05 | 4 |
| $06 | 80 |
| $07 | 6 |
| $08 | 160 |
| $09 | 8 |
| $0A | 60 |
| $0B | 10 |
| $0C | 14 |
| $0D | 12 |
| $0E | 26 |
| $0F | 14 |
| $10 | 12 |
| $11 | 16 |
| $12 | 24 |
| $13 | 18 |
| $14 | 48 |
| $15 | 20 |
| $16 | 96 |
| $17 | 22 |
| $18 | 192 |
| $19 | 24 |
| $1A | 72 |
| $1B | 26 |
| $1C | 16 |
| $1D | 28 |
| $1E | 32 |
| $1F | 30 |

### Table structure

The structure of the length table becomes clearer when rearranged like in the following index to length map (which corresponds to the order in which the values appear in the [internal APU lookup table](https://www.nesdev.org/wiki/Visual_circuit_tutorial#Decoders_and_mask_ROMs)):

```
Legend:
<bit pattern> (<value of bit pattern>) => <note length>

Linear length values:
1 1111 (1F) => 30
1 1101 (1D) => 28
1 1011 (1B) => 26
1 1001 (19) => 24
1 0111 (17) => 22
1 0101 (15) => 20
1 0011 (13) => 18
1 0001 (11) => 16
0 1111 (0F) => 14
0 1101 (0D) => 12
0 1011 (0B) => 10
0 1001 (09) => 8
0 0111 (07) => 6
0 0101 (05) => 4
0 0011 (03) => 2
0 0001 (01) => 254

Notes with base length 12 (4/4 at 75 bpm):
1 1110 (1E) => 32  (96 times 1/3, quarter note triplet)
1 1100 (1C) => 16  (48 times 1/3, eighth note triplet)
1 1010 (1A) => 72  (48 times 1 1/2, dotted quarter)
1 1000 (18) => 192 (Whole note)
1 0110 (16) => 96  (Half note)
1 0100 (14) => 48  (Quarter note)
1 0010 (12) => 24  (Eighth note)
1 0000 (10) => 12  (Sixteenth)

Notes with base length 10 (4/4 at 90 bpm, with relative durations being the same as above):
0 1110 (0E) => 26  (Approx. 80 times 1/3, quarter note triplet)
0 1100 (0C) => 14  (Approx. 40 times 1/3, eighth note triplet)
0 1010 (0A) => 60  (40 times 1 1/2, dotted quarter)
0 1000 (08) => 160 (Whole note)
0 0110 (06) => 80  (Half note)
0 0100 (04) => 40  (Quarter note)
0 0010 (02) => 20  (Eighth note)
0 0000 (00) => 10  (Sixteenth)
```

With the least significant bit set, the remaining bits select a linear length (with the exception of the 0 entry). Otherwise, we get note lengths based on a base length of 10 (MSB clear) or 12 (MSB set).

<table>
<tr><th colspan="2"><div><a href="https://www.nesdev.org/wiki/Template:APU_nav">V</a> | T</div> <a href="APU.md">APU</a></th></tr>
<tr><td><b>Concepts</b></td><td><a href="APU_registers.md">Registers</a> | <a href="APU_basics.md">Basics</a> | <a href="APU_period_table.md">Period table</a></td></tr>
<tr><td><b>Channels</b></td><td><a href="APU_Pulse.md">Pulse 1 and Pulse 2</a> | <a href="APU_Triangle.md">Triangle</a> | <a href="APU_Noise.md">Noise</a> | <a href="APU_DMC.md">DMC</a></td></tr>
<tr><td><b>Components</b></td><td><a href="APU_Envelope.md">Envelope</a> | <a href="APU_Sweep.md">Sweep</a> | <a href="APU_Frame_Counter.md">Frame Counter</a> | <a>Length Counter</a> | <a href="DMA.md">DMA</a> | <a href="APU_Mixer.md">Mixer</a></td></tr>
<tr><td><b>Expansion audio</b></td><td><a href="https://www.nesdev.org/wiki/FDS_audio">2C33 (Disk System)</a> | <a href="https://www.nesdev.org/wiki/MMC5_audio">MMC5</a> | <a href="https://www.nesdev.org/wiki/VRC6_audio">VRC6</a> | <a href="https://www.nesdev.org/wiki/VRC7_audio">VRC7</a> | <a href="https://www.nesdev.org/wiki/Namco_163_audio">Namco 163</a> | <a href="https://www.nesdev.org/wiki/Sunsoft_5B_audio">Sunsoft 5B</a> | <a href="https://www.nesdev.org/wiki/INES_Mapper_086#Registers">µPD7756</a></td></tr></table>
