# APU DMC

> Source: <https://www.nesdev.org/wiki/APU_DMC> — NESdev Wiki, revision [24164](https://www.nesdev.org/w/index.php?oldid=24164) (last edited 2026-08-29T02:03:21Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=APU_DMC&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The [NES APU's](APU.md) delta modulation channel (DMC) can output 1-bit [delta-encoded samples](https://en.wikipedia.org/wiki/Delta_modulation) or can have its 7-bit counter directly loaded, allowing flexible manual sample playback.

## Overview

The DMC channel contains the following: memory reader, interrupt flag, sample buffer, [timer](APU.md#Glossary), output unit, 7-bit output level with up and down counter.

```
                         Timer
                           |
                           v
Reader ---> Buffer ---> Shifter ---> Output level ---> (to the mixer)
```

<table>
<tr><td><b>$4010</b></td><td><tt>IL--.RRRR</tt></td><td><b>Flags and Rate</b> (write)</td></tr>
<tr><td>bit 7</td><td><tt>I---.----</tt></td><td>IRQ enabled flag. If clear, the interrupt flag is cleared.</td></tr>
<tr><td>bit 6</td><td><tt>-L--.----</tt></td><td>Loop flag</td></tr>
<tr><td>bits 3-0</td><td><tt>----.RRRR</tt></td><td>Rate index<br/>
<pre>Rate   $0   $1   $2   $3   $4   $5   $6   $7   $8   $9   $A   $B   $C   $D   $E   $F
      ------------------------------------------------------------------------------
NTSC  428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54
PAL   398, 354, 316, 298, 276, 236, 210, 198, 176, 148, 132, 118,  98,  78,  66,  50
</pre>
<p>The rate determines for how many CPU cycles happen between changes in the output level during automatic delta-encoded sample playback. For example, on NTSC (1.789773 MHz), a rate of 428 gives a frequency of 1789773/428 Hz = 4181.71 Hz. These periods are all even numbers because there are 2 CPU cycles in an APU cycle. A rate of 428 means the output level changes every 214 APU cycles.
</p></td></tr>
<tr><td colspan="3"></td></tr>
<tr><td><b>$4011</b></td><td><tt>-DDD.DDDD</tt></td><td><b>Direct load</b> (write)</td></tr>
<tr><td>bits 6-0</td><td><tt>-DDD.DDDD</tt></td><td>The DMC output level is set to D, an unsigned value. If the timer is outputting a clock at the same time, the output level is occasionally not changed properly.<a href="https://forums.nesdev.org/viewtopic.php?p=104491#p104491">[1]</a></td></tr>
<tr><td colspan="3"></td></tr>
<tr><td><b>$4012</b></td><td><tt>AAAA.AAAA</tt></td><td><b>Sample address</b> (write)</td></tr>
<tr><td>bits 7-0</td><td><tt>AAAA.AAAA</tt></td><td>Sample address = <tt>%11AAAAAA.AA000000</tt> = <tt>$C000 + (A * 64)</tt></td></tr>
<tr><td colspan="3"></td></tr>
<tr><td><b>$4013</b></td><td><tt>LLLL.LLLL</tt></td><td><b>Sample length</b> (write)</td></tr>
<tr><td>bits 7-0</td><td><tt>LLLL.LLLL</tt></td><td>Sample length = <tt>%LLLL.LLLL0001</tt> = <tt>(L * 16) + 1 bytes</tt></td></tr></table>

The output level is sent to the [mixer](APU_Mixer.md) whether the channel is enabled or not. It is loaded with 0 on power-up, and can be updated by $4011 writes and delta-encoded sample playback.

Automatic 1-bit [delta-encoded sample](https://en.wikipedia.org/wiki/Delta_modulation) playback is carried out by a combination of three units. The *memory reader* fills the 8-bit *sample buffer* whenever it is emptied by the sample *output unit*. The [status register](APU.md#Status_($4015)) is used to start and stop automatic sample playback.

The **sample buffer** either holds a single 8-bit sample byte or is empty. It is filled by the reader and can only be emptied by the output unit; once loaded with a sample byte it will be played back.

### Pitch table

<table>
<tr><td></td><td colspan="6">NTSC</td><td colspan="6">PAL</td></tr>
<tr><th>$4010</th><th>Period</th><th>Frequency</th><th>Note (raw)</th><th>Note (1-byte loop)</th><th>Note (17-byte loop)</th><th>Note (33-byte loop)</th><th>Period</th><th>Frequency</th><th>Note (raw)</th><th>Note (1-byte loop)</th><th>Note (17-byte loop)</th><th>Note (33-byte loop)</th></tr>
<tr><td>$0</td><td>$1AC</td><td>4181.71 Hz</td><td>C-8  -2c</td><td>C-5 -2c</td><td>B-0 -7c</td><td>infrasound</td><td>$18E</td><td>4177.40 Hz</td><td>C-8  -4c</td><td>C-5 -4c</td><td>B-0 -9c</td><td>infrasound</td></tr>
<tr><td>$1</td><td>$17C</td><td>4709.93 Hz</td><td>D-8  +4c</td><td>D-5 +4c</td><td>C#1 -1c</td><td>infrasound</td><td>$162</td><td>4696.63 Hz</td><td>D-8  -1c</td><td>D-5 -1c</td><td>C#1 -6c</td><td>infrasound</td></tr>
<tr><td>$2</td><td>$154</td><td>5264.04 Hz</td><td>E-8  -3c</td><td>E-5 -3c</td><td>Eb1 -8c</td><td>infrasound</td><td>$13C</td><td>5261.41 Hz</td><td>E-8  -4c</td><td>E-5 -4c</td><td>Eb1 -9c</td><td>infrasound</td></tr>
<tr><td>$3</td><td>$140</td><td>5593.04 Hz</td><td>F-8  +2c</td><td>F-5 +2c</td><td>E-1 -3c</td><td>E-0 +48c</td><td>$12A</td><td>5579.22 Hz</td><td>F-8  -3c</td><td>F-5 -3c</td><td>E-1 -8c</td><td>E-0 +44c</td></tr>
<tr><td>$4</td><td>$11E</td><td>6257.95 Hz</td><td>G-8  -4c</td><td>G-5 -4c</td><td>F#1 -9c</td><td>F#0 +43c</td><td>$114</td><td>6023.94 Hz</td><td>G-8  -70c</td><td>G-5 -70c</td><td>F#1 -75c</td><td>F#0 -23c</td></tr>
<tr><td>$5</td><td>$0FE</td><td>7046.35 Hz</td><td>A-8  +2c</td><td>A-5 +2c</td><td>Ab1 -3c</td><td>Ab0 +48c</td><td>$0EC</td><td>7044.94 Hz</td><td>A-8  +1c</td><td>A-5 +1c</td><td>Ab1 -4c</td><td>Ab0 +48c</td></tr>
<tr><td>$6</td><td>$0E2</td><td>7919.35 Hz</td><td>B-8  +4c</td><td>B-5 +4c</td><td>Bb1 -1c</td><td>Bb0 +50c</td><td>$0D2</td><td>7917.18 Hz</td><td>B-8  +3c</td><td>B-5 +3c</td><td>Bb1 -2c</td><td>Bb0 +50c</td></tr>
<tr><td>$7</td><td>$0D6</td><td>8363.42 Hz</td><td>C-9  -2c</td><td>C-6 -2c</td><td>B-1 -7c</td><td>B-0 +45c</td><td>$0C6</td><td>8397.01 Hz</td><td>C-9  +5c</td><td>C-6 +5c</td><td>B-1 +0c</td><td>B-0 +52c</td></tr>
<tr><td>$8</td><td>$0BE</td><td>9419.86 Hz</td><td>D-9  +4c</td><td>D-6 +4c</td><td>C#2 -1c</td><td>C#1 +51c</td><td>$0B0</td><td>9446.63 Hz</td><td>D-9  +9c</td><td>D-9 +9c</td><td>C#2 +4c</td><td>C#1 +56c</td></tr>
<tr><td>$9</td><td>$0A0</td><td>11186.1 Hz</td><td>F-9  +2c</td><td>F-6 +2c</td><td>E-2 -3c</td><td>E-1 +48c</td><td>$094</td><td>11233.8 Hz</td><td>F-9  +9c</td><td>F-6 +9c</td><td>E-2 +4c</td><td>E-1 +56c</td></tr>
<tr><td>$A</td><td>$08E</td><td>12604.0 Hz</td><td>G-9  +8c</td><td>G-6 +8c</td><td>F#2 +3c</td><td>F#1 +55c</td><td>$084</td><td>12595.5 Hz</td><td>G-9  +7c</td><td>G-6 +7c</td><td>F#2 -2c</td><td>G-1 +54c</td></tr>
<tr><td>$B</td><td>$080</td><td>13982.6 Hz</td><td>A-9  -12c</td><td>A-6 -12c</td><td>Ab2 -17c</td><td>Ab1 +35c</td><td>$076</td><td>14089.9 Hz</td><td>A-9  +1c</td><td>A-6 +1c</td><td>Ab2 -4c</td><td>Ab1 +48c</td></tr>
<tr><td>$C</td><td>$06A</td><td>16884.6 Hz</td><td>C-10 +14c</td><td>C-7 +14c</td><td>B-2 +10c</td><td>B-1 +61c</td><td>$062</td><td>16965.4 Hz</td><td>C-10 +23c</td><td>C-7 +23c</td><td>B-2 +18c</td><td>B-1 +69c</td></tr>
<tr><td>$D</td><td>$054</td><td>21306.8 Hz</td><td>E-10 +17c</td><td>E-7 +17c</td><td>Eb3 +12c</td><td>Eb2 +64c</td><td>$04E</td><td>21315.5 Hz</td><td>E-10 +18c</td><td>E-7 +18c</td><td>Eb3 +13c</td><td>Eb2 +65c</td></tr>
<tr><td>$E</td><td>$048</td><td>24858.0 Hz</td><td>G-10 -16c</td><td>G-7 -16c</td><td>F#3 -21c</td><td>F#2 +31c</td><td>$042</td><td>25191.0 Hz</td><td>G-10 +7c</td><td>G-7 +7c</td><td>F#3 +2c</td><td>F#2 +54c</td></tr>
<tr><td>$F</td><td>$036</td><td>33143.9 Hz</td><td>C-11 -18c</td><td>C-8 -18c</td><td>B-3 -23c</td><td>B-2 +29c</td><td>$032</td><td>33252.1 Hz</td><td>C-11 -12c</td><td>C-8 -12c</td><td>B-3 -17c</td><td>B-2 +34c</td></tr></table>

(Deviation from note is given in cents, which are defined as 1/100th of a semitone.)

Note that on PAL systems, the pitches at $4 and $C appear to be incorrect with respect to their intended A-440 tuning scheme[[1]](#cite_note-1).

### Memory reader

When the sample buffer is emptied, the memory reader fills the sample buffer with the next byte from the currently playing sample. It has an address counter and a bytes remaining counter.

When a sample is (re)started, the current address is set to the sample address, and bytes remaining is set to the sample length.

Any time the sample buffer is in an empty state and bytes remaining is not zero (including just after a write to $4015 that enables the channel, regardless of where that write occurs relative to the bit counter mentioned below), the following occur:

* The [CPU](CPU.md) is stalled for 1-4 CPU cycles to read a sample byte. The exact cycle count depends on many factors and is described in detail in the [DMA](DMA.md) article.
* The sample buffer is filled with the next sample byte read from the current address, subject to whatever [mapping hardware](Mapper.md) is present.
* The address is incremented; if it exceeds $FFFF, it is wrapped around to $8000.
* The bytes remaining counter is decremented; if it becomes zero and the loop flag is set, the sample is restarted (see above); otherwise, if the bytes remaining counter becomes zero and the IRQ enabled flag is set, the interrupt flag is set.

At any time, if the interrupt flag is set, the [CPU's IRQ line](CPU.md) is *continuously* asserted until the interrupt flag is cleared.
The processor will continue on from where it was stalled. The IRQ can be disabled via writing any values to $4015 or clearing interrupt flag through $4010.7.

### Output unit

The output unit continuously outputs a 7-bit value to the [mixer](APU_Mixer.md). It contains an 8-bit right shift register, a bits-remaining counter, a 7-bit output level (the same one that can be loaded directly via $4011), and a silence flag.

The bits-remaining counter is updated whenever the [timer](APU.md#Glossary) outputs a clock, regardless of whether a sample is currently playing. When this counter reaches zero, we say that the output cycle ends. The DPCM unit can only transition from silent to playing at the end of an output cycle.

When an output cycle ends, a new cycle is started as follows:

* The bits-remaining counter is loaded with 8.
* If the sample buffer is empty, then the silence flag is set; otherwise, the silence flag is cleared and the sample buffer is emptied into the shift register.

When the timer outputs a clock, the following actions occur in order:

1. If the silence flag is clear, the output level changes based on bit 0 of the shift register. If the bit is 1, add 2; otherwise, subtract 2. But if adding or subtracting 2 would cause the output level to leave the 0-127 range, leave the output level unchanged. This means subtract 2 only if the current level is at least 2, or add 2 only if the current level is at most 125.
2. The right shift register is clocked.
3. As stated above, the bits-remaining counter is decremented. If it becomes zero, a new output cycle is started.

*Nothing can interrupt a cycle; every cycle runs to completion before a new cycle is started.*

The shift register shifts to the right. This means the channel plays the least significant bits of each byte before playing the most significant bits. Nintendo games' samples are encoded with the correct bit order. Some third-party games (e.g. *Double Dribble*) have their bits reversed, which makes the audio less clear.[[2]](#cite_note-2)

## Conflict with controller and PPU read

On the 2A03 CPU used in the NTSC NES and Famicom, if a new sample byte is fetched from memory at the same time the program is reading a register that has read side effects (such as [controllers via $4016 or $4017](Controller_reading.md), [PPU data via $2007](PPU_registers.md#PPUDATA), or [the vblank flag via $2002](PPU_registers.md#PPUSTATUS)), that register will see multiple extra reads that result in data loss or corruption. This is mostly a problem for controllers, and special [controller reading code](Controller_reading_code.md#DPCM_Safety) is necessary to work around this.

This problem is fixed on the 2A07 series of PAL CPUs by forcing DMA to halt on the first cycle of an instruction, which fetches an instruction opcode and thus isn't normally reading from registers.

See [DMA](DMA.md) for a thorough explanation of how DMA works and [how it conflicts with register reads](DMA.md#Register_conflicts).

## Usage of DMC for syncing to video

DMC IRQs can be used for timed video operations. The following method was discussed on the forum in 2010.[[3]](#cite_note-3)

### Concept

The NES hardware only has limited tools for syncing the code with video rendering. The VBlank NMI and sprite 0 hit are the only two reasonably reliable flags that can be used, so only 2 synchronizations per frame can be done easily. In addition, only the VBlank NMI can trigger an interrupt; the sprite 0 hit flag has to be polled, potentially wasting a lot of CPU cycles.

However, the DMC channel can hypothetically be used for syncing with video instead of using it for sound. Unfortunately it's a bit complicated, but used correctly, it can function as a crude scanline counter, eliminating the need for an advanced mapper.

The DMC's timing is completely separate from the video. The DMC's timer is always running, and samples can only start every 8 clock cycles. However, because the DMC's timer isn't synchronized to the PPU in any way, these 8-clock boundaries occur on different scanlines each frame.

Here are the steps to achieve stable timing:

* At a fixed point in video rendering (we'll use the start of vblank as an example), a dummy single-byte sample at rate $F is started. Due to a hardware quirk†, the sample needs to be started three times in a row like this:

```
sei
lda #$10
sta $4015
sta $4015
sta $4015
cli
```

* The amount of cycles before a DMC IRQ happens is then measured (either using an actual IRQ, or by polling $4015).
  * At rate $F, there are 54 CPU cycles between clocks, so there are 432 CPU cycles (432 × 3 ÷ 341 = about 3.8 scanlines) between boundaries.
* The main sample that will be used for the timing is then started (please refer to the table below to have sample lengths for various waiting times)
* When the main IRQ happens, the measurement from before is retrieved, and a timing loop with variable delay is used. In order to synchronize with vblank, after a DMC IRQ we should wait 432 CPU cycles minus the time we measured.

†**Note:** The hardware quirk mentioned above deals with how DMC IRQs are generated. Basically, the IRQ is generated when the last **byte** of the sample is **read**, **not** when the last *sample* of the sample *plays*. The sample buffer sometimes has enough time to empty itself between writes to $4015, meaning your next write to $4015 will trigger an immediate IRQ. [Fortunately, writing to $4015 three times will avoid this issue.](https://forums.nesdev.org/viewtopic.php?p=66712#p66712)

Still using vblank as an example, the measurement tells how far into the 8-clock boundary vblank occurred, and by delaying after a DMC IRQ, we perform a raster effect at the same point within the 8-clock boundary, aligning it with vblank. By performing this same method each frame, the raster effect will have a reasonably stable timing to it. As a bonus, since mostly using IRQs are being used, the CPU is free to do something else, instead of waiting in a timed loop.

It's possible to use more than one IRQ per frame - but the *measurement* part needs to be done at the *same time* within each frame, before the usage of any IRQ.

Only a single split-point per IRQ is possible, with the shortest IRQ being 3.8 scanlines. For split points closer than this amount, timed code has to be used.

In order to remain silent, samples should be made up of all $00 bytes, and $00 should have been previously written to $4011. Otherwise, audio will unintentionally be created. This *is* a sound channel, after all.

### Timing table

This table converts sample length in scanline length (all values are rounded to the higher integer).

```
NTSC               Rate
Length              $0    $1   $2   $3   $4   $5   $6   $7   $8   $9   $a   $b   $c   $d   $e   $f
----------------------------------------------------------------------------------------------------
1-byte (8 bits)     31    27   24   23   21   18   16   16   14   12   10   10   8    6    6    4
17-byte (136 bits)  **    **   **   **   **   **   **   **   228  192  170  154  127  101  87   65
33-byte (264 bits)  **    **   **   **   **   **   **   **   **   **   **   **   **   196  168  126
49-byte (392 bits)  **    **   **   **   **   **   **   **   **   **   **   **   **   **   **   187

PAL                Rate
Length              $0    $1   $2   $3   $4   $5   $6   $7   $8   $9   $a   $b   $c   $d   $e   $f
----------------------------------------------------------------------------------------------------
1-byte (8 bits)     30    27   24   23   21   18   16   15   14   12   10   9    8    6    5    4
17-byte (136 bits)  **    **   **   **   **   **   **   **   225  189  169  151  126  100  85   64
33-byte (264 bits)  **    **   **   **   **   **   **   **   **   **   **   **   **   194  164  124
49-byte (392 bits)  **    **   **   **   **   **   **   **   **   **   **   **   **   **   **   184
```

### Number of scanlines to wait table

This table gives the best sample length and frequency combinations for all possible scanlines interval to wait. They are best because they are where the CPU will have to kill the least time. However, it's still possible to use options to wait for fewer lines and kill more time during the interrupt before the video effect.

Because a PAL interrupt will always happen about the same time or a bit sooner than a NTSC interrupt, the NTSC table will be used to set the "best" setting here :

```
Scanlines  Best opt. for IRQ

1-3        Timed code
4-5        Length $0, rate $f
6-7        Length $0, rate $d
8-9        Length $0, rate $c
10-11      Length $0, rate $a
12-13      Length $0, rate $9
14-15      Length $0, rate $8
16-17      Length $0, rate $6
18-20      Length $0, rate $5
21-22      Length $0, rate $4
23         Length $0, rate $3
24-26      Length $0, rate $2
27-30      Length $0, rate $1
31-64      Length $0, rate $0
65-86      Length $1, rate $f
87-100     Length $1, rate $e
101-125    Length $1, rate $d
126        Length $2, rate $f
127-153    Length $1, rate $c
154-167    Length $1, rate $b
168-169    Length $2, rate $e
170-186    Length $1, rate $a
187-191    Length $3, rate $f
192-195    Length $1, rate $9
196-227    Length $2, rate $d
228-239    Length $1, rate $8
```

## References

1. [↑](#cite_ref-1) [Forum post](https://forums.nesdev.org/viewtopic.php?p=94079#p94079): PAL DPCM frequency table contains 2 errors.
2. [↑](#cite_ref-2) [Forum thread](https://forums.nesdev.org/viewtopic.php?t=20308): DPCM bit-reversal in third-party games
3. [↑](#cite_ref-3) [Forum thread](https://forums.nesdev.org/viewtopic.php?t=6521): DMC IRQ as a video timer.

<table>
<tr><th colspan="2"><div><a href="https://www.nesdev.org/wiki/Template:APU_nav">V</a> | T</div> <a href="APU.md">APU</a></th></tr>
<tr><td><b>Concepts</b></td><td><a href="APU_registers.md">Registers</a> | <a href="APU_basics.md">Basics</a> | <a href="APU_period_table.md">Period table</a></td></tr>
<tr><td><b>Channels</b></td><td><a href="APU_Pulse.md">Pulse 1 and Pulse 2</a> | <a href="APU_Triangle.md">Triangle</a> | <a href="APU_Noise.md">Noise</a> | <a>DMC</a></td></tr>
<tr><td><b>Components</b></td><td><a href="APU_Envelope.md">Envelope</a> | <a href="APU_Sweep.md">Sweep</a> | <a href="APU_Frame_Counter.md">Frame Counter</a> | <a href="APU_Length_Counter.md">Length Counter</a> | <a href="DMA.md">DMA</a> | <a href="APU_Mixer.md">Mixer</a></td></tr>
<tr><td><b>Expansion audio</b></td><td><a href="https://www.nesdev.org/wiki/FDS_audio">2C33 (Disk System)</a> | <a href="https://www.nesdev.org/wiki/MMC5_audio">MMC5</a> | <a href="https://www.nesdev.org/wiki/VRC6_audio">VRC6</a> | <a href="https://www.nesdev.org/wiki/VRC7_audio">VRC7</a> | <a href="https://www.nesdev.org/wiki/Namco_163_audio">Namco 163</a> | <a href="https://www.nesdev.org/wiki/Sunsoft_5B_audio">Sunsoft 5B</a> | <a href="https://www.nesdev.org/wiki/INES_Mapper_086#Registers">µPD7756</a></td></tr></table>
