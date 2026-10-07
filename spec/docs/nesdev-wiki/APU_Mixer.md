# APU Mixer

> Source: <https://www.nesdev.org/wiki/APU_Mixer> — NESdev Wiki, revision [23451](https://www.nesdev.org/w/index.php?oldid=23451) (last edited 2026-01-19T03:49:41Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=APU_Mixer&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The [NES APU](APU.md) mixer takes the channel outputs and converts them to an analog audio signal. Each channel has its own internal digital-to-analog convertor (DAC), implemented in a way that causes non-linearity and interaction between channels, so calculation of the resulting amplitude is somewhat involved. In particular, games such as *Super Mario Bros.* and *StarTropics* use the DMC level ($4011) as a crude volume control for the triangle and noise channels.

The following formula[[1]](#cite_note-1) calculates the approximate audio output level within the range of 0.0 to 1.0. It is the sum of two sub-groupings of the channels:

```
output = pulse_out + tnd_out

                            95.88
pulse_out = ------------------------------------
             (8128 / (pulse1 + pulse2)) + 100

                                       159.79
tnd_out = -------------------------------------------------------------
                                    1
           ----------------------------------------------------- + 100
            (triangle / 8227) + (noise / 12241) + (dmc / 22638)
```

The values for [pulse1](APU_Pulse.md), [pulse2](APU_Pulse.md), [triangle](APU_Triangle.md), [noise](APU_Noise.md), and [dmc](APU_DMC.md) are the output values for the corresponding channel. The dmc value ranges from 0 to 127 and the others range from 0 to 15. When the values for one of the groups are all zero, the result for that group should be treated as zero rather than undefined due to the division by 0 that otherwise results.

Faster but less accurate approximations are also possible: using an efficient [lookup table](#Lookup_Table), or even rougher with a [linear formula](#Linear_Approximation).

The NES hardware follows the DACs with a [surprisingly involved circuit](https://archive.nesdev.org/NESAudio.gif) that adds several low-pass and high-pass filters:

* A first-order high-pass filter at 90 Hz
* Another first-order high-pass filter at 440 Hz
* A first-order low-pass filter at 14 kHz

See also:

* [blargg's data and analysis, and lidnariq's matching analog components to filters](http://forums.nesdev.org/viewtopic.php?p=44255#p44255)

The Famicom hardware instead ONLY specifies a first-order high-pass filter at 37 Hz, followed by the unknown (and varying) properties of the RF modulator and demodulator.

## Emulation

The NES APU Mixer can be efficiently emulated using a lookup table or a less-accurate linear approximation.

### Lookup Table

The APU mixer formulas can be efficiently implemented using two lookup tables: a 31-entry table for the two [pulse channels](APU_Pulse.md) and a 203-entry table for the remaining channels (due to the approximation of tnd_out, the numerators are adjusted slightly to preserve the normalized output range).

```
    output = pulse_out + tnd_out

    pulse_table [n] = 95.52 / (8128.0 / n + 100)

    pulse_out = pulse_table [pulse1 + pulse2]
```

The tnd_out table is approximated (within 4%) by using a base unit close to the [DMC's](APU_DMC.md) DAC.

```
    tnd_table [n] = 163.67 / (24329.0 / n + 100)

    tnd_out = tnd_table [3 * triangle + 2 * noise + dmc]
```

### Linear Approximation

A linear approximation can also be used, which results in slightly louder DMC samples, but otherwise fairly accurate operation since the wave channels use a small portion of the transfer curve. The overall volume will be reduced due to the headroom required by the DMC approximation.

```
    output = pulse_out + tnd_out

    pulse_out = 0.00752 * (pulse1 + pulse2)

    tnd_out = 0.00851 * triangle + 0.00494 * noise + 0.00335 * dmc
```

## References

1. [↑](#cite_ref-1) [apu_ref.txt](https://www.nesdev.org/apu_ref.txt) by blargg

<table>
<tr><th colspan="2"><div><a href="https://www.nesdev.org/wiki/Template:APU_nav">V</a> | T</div> <a href="APU.md">APU</a></th></tr>
<tr><td><b>Concepts</b></td><td><a href="APU_registers.md">Registers</a> | <a href="APU_basics.md">Basics</a> | <a href="APU_period_table.md">Period table</a></td></tr>
<tr><td><b>Channels</b></td><td><a href="APU_Pulse.md">Pulse 1 and Pulse 2</a> | <a href="APU_Triangle.md">Triangle</a> | <a href="APU_Noise.md">Noise</a> | <a href="APU_DMC.md">DMC</a></td></tr>
<tr><td><b>Components</b></td><td><a href="APU_Envelope.md">Envelope</a> | <a href="APU_Sweep.md">Sweep</a> | <a href="APU_Frame_Counter.md">Frame Counter</a> | <a href="APU_Length_Counter.md">Length Counter</a> | <a href="DMA.md">DMA</a> | <a>Mixer</a></td></tr>
<tr><td><b>Expansion audio</b></td><td><a href="https://www.nesdev.org/wiki/FDS_audio">2C33 (Disk System)</a> | <a href="https://www.nesdev.org/wiki/MMC5_audio">MMC5</a> | <a href="https://www.nesdev.org/wiki/VRC6_audio">VRC6</a> | <a href="https://www.nesdev.org/wiki/VRC7_audio">VRC7</a> | <a href="https://www.nesdev.org/wiki/Namco_163_audio">Namco 163</a> | <a href="https://www.nesdev.org/wiki/Sunsoft_5B_audio">Sunsoft 5B</a> | <a href="https://www.nesdev.org/wiki/INES_Mapper_086#Registers">µPD7756</a></td></tr></table>
