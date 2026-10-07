# APU Sweep

> Source: <https://www.nesdev.org/wiki/APU_Sweep> — NESdev Wiki, revision [24338](https://www.nesdev.org/w/index.php?oldid=24338) (last edited 2026-10-02T00:47:49Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=APU_Sweep&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

An [NES APU](APU.md) sweep unit can be made to periodically adjust a [pulse channel](APU_Pulse.md)'s period up or down.

## Sweep registers ($4001 / $4005, write)

* $4001 corresponds to [Pulse 1](APU_Pulse.md#Sweep_control_($4001_/_$4005,_write))'s sweep unit
* $4005 corresponds to [Pulse 2](APU_Pulse.md#Sweep_control_($4001_/_$4005,_write))'s sweep unit

```
7  bit  0
---- ----
EPPP NSSS
|||| ||||
|||| |+++- Bit shift count
|||| +---- Negate flag
||||           0: add to period, sweeping toward lower frequencies
||||           1: subtract from period, sweeping toward higher frequencies
|+++------ Clock counter divider value
+--------- Enable flag
```

Writing to this register also sets reload flag.

## Updating the period

When the [frame counter](APU_Frame_Counter.md) generates a half-frame clock (at 120 or 96 Hz), two things occur:

1. If the clock divider's counter *is* zero, the sweep is enabled (via [E flag](#Sweep_registers_($4001_/_$4005,_write))), the shift count ([S bits](#Sweep_registers_($4001_/_$4005,_write))) is nonzero,
   1. and the sweep unit is *not* [muting](#Muting) the channel: the pulse's period is set to the target period.
   2. and the sweep unit *is* [muting](#Muting) the channel: the pulse's period remains *unchanged*, but the sweep unit's divider continues to count down and reload the divider's period as normal.
2. If the clock divider's counter *is* zero *or* the reload flag is true: The divider counter is set to P and the reload flag is cleared. Otherwise, the divider counter is decremented.

If the sweep unit is disabled including because the shift count is zero, the pulse channel's period is never updated, but muting logic still applies.

## Calculating the target period

The sweep unit continuously calculates each pulse channel's **target period** in this way:

1. A barrel shifter shifts the pulse channel's 11-bit [raw timer period](APU_Pulse.md) right by the shift count ([S bits](#Sweep_registers_($4001_/_$4005,_write))), producing the change amount.
2. If the negate flag ([N bit](#Sweep_registers_($4001_/_$4005,_write))) is true, the change amount is made negative.
3. The target period is the sum of the current period and the change amount, clamped to zero if this sum is negative.

For example, if the negate flag ([N bit](#Sweep_registers_($4001_/_$4005,_write))) is false and the shift count ([S bits](#Sweep_registers_($4001_/_$4005,_write))) is zero, the change amount equals the current period, making the target period equal to *twice* the current period.

The two pulse channels have their adders' carry inputs wired differently, which produces different results when each channel's change amount is made negative:

* Pulse 1 adds the [ones' complement](https://en.wikipedia.org/wiki/Ones%27_complement) (*−c − 1*). Making 20 negative produces a change amount of −21.
* Pulse 2 adds the [two's complement](https://en.wikipedia.org/wiki/Two%27s_complement) (*−c*). Making 20 negative produces a change amount of −20.

Whenever the current period or sweep setting changes, whether by [$400x](APU_Pulse.md#Registers) writes or by sweep updating the period, the target period also changes.

## Muting

**Muting** means that the pulse channel sends 0 to the [mixer](APU_Mixer.md) instead of the current volume. Muting happens regardless of whether **the sweep unit is disabled** and regardless of whether the sweep divider is outputting a clock signal.

Two conditions cause the sweep unit to mute the channel until the condition ends:

1. If the *current* period is less than 8, the sweep unit mutes the channel.
2. If at any time the *target* period is greater than $7FF, the sweep unit mutes the channel.
   * In particular, if the negate flag ([N bit](#Sweep_registers_($4001_/_$4005,_write))) is false, the shift count is zero, and the current period is at least $400, the target period will be large enough to mute the channel. (This is why several publishers' NES games never seem to use the bottom octave of the pulse waves.)

Because the target period is computed continuously, a target period overflow from the sweep unit's adder can silence a channel **even when the sweep unit is disabled** and even when the sweep divider is not outputting a clock signal.

Thus to fully disable the sweep unit, a program must additionally turn on the negate flag ([N bit](#Sweep_registers_($4001_/_$4005,_write))), such as by writing $08.
This ensures that the target period is not greater than the current period and therefore not greater than $7FF.

<table>
<tr><th colspan="2"><div><a href="https://www.nesdev.org/wiki/Template:APU_nav">V</a> | T</div> <a href="APU.md">APU</a></th></tr>
<tr><td><b>Concepts</b></td><td><a href="APU_registers.md">Registers</a> | <a href="APU_basics.md">Basics</a> | <a href="APU_period_table.md">Period table</a></td></tr>
<tr><td><b>Channels</b></td><td><a href="APU_Pulse.md">Pulse 1 and Pulse 2</a> | <a href="APU_Triangle.md">Triangle</a> | <a href="APU_Noise.md">Noise</a> | <a href="APU_DMC.md">DMC</a></td></tr>
<tr><td><b>Components</b></td><td><a href="APU_Envelope.md">Envelope</a> | <a>Sweep</a> | <a href="APU_Frame_Counter.md">Frame Counter</a> | <a href="APU_Length_Counter.md">Length Counter</a> | <a href="DMA.md">DMA</a> | <a href="APU_Mixer.md">Mixer</a></td></tr>
<tr><td><b>Expansion audio</b></td><td><a href="https://www.nesdev.org/wiki/FDS_audio">2C33 (Disk System)</a> | <a href="https://www.nesdev.org/wiki/MMC5_audio">MMC5</a> | <a href="https://www.nesdev.org/wiki/VRC6_audio">VRC6</a> | <a href="https://www.nesdev.org/wiki/VRC7_audio">VRC7</a> | <a href="https://www.nesdev.org/wiki/Namco_163_audio">Namco 163</a> | <a href="https://www.nesdev.org/wiki/Sunsoft_5B_audio">Sunsoft 5B</a> | <a href="https://www.nesdev.org/wiki/INES_Mapper_086#Registers">µPD7756</a></td></tr></table>
