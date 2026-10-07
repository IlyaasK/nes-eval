# APU Noise

> Source: <https://www.nesdev.org/wiki/APU_Noise> — NESdev Wiki, revision [24354](https://www.nesdev.org/w/index.php?oldid=24354) (last edited 2026-10-05T10:45:53Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=APU_Noise&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The [NES APU](APU.md) noise channel generates pseudo-random 1-bit noise at 16 different frequencies.

```
   Timer --> Shift Register   Length Counter
                   |                |
                   v                v
Envelope -------> Gate ----------> Gate ---> (to mixer)
```

# Registers

## Noise envelope settings ($400C, write)

```
7  bit  0
---- ----
..LC VVVV
  || ||||
  || ++++- When C is set: channel volume value
  ||       When C is clear: envelope divider reload value
  |+------ Constant volume flag (override envelope with V)
  +------- Length counter halt flag and envelope loop flag
```

## Noise mode and period ($400E, write)

```
7  bit  0
---- ----
M... PPPP
|    ||||
|    ++++- Noise period
+--------- Periodic noise mode
```

The timer period is set to entry P of the following:

| Period (P) | NTSC | PAL |
|---|---|---|
| $0 | 4 | 4 |
| $1 | 8 | 8 |
| $2 | 16 | 14 |
| $3 | 32 | 30 |
| $4 | 64 | 60 |
| $5 | 96 | 88 |
| $6 | 128 | 118 |
| $7 | 160 | 148 |
| $8 | 202 | 188 |
| $9 | 254 | 236 |
| $A | 380 | 354 |
| $B | 508 | 472 |
| $C | 762 | 708 |
| $D | 1016 | 944 |
| $E | 2034 | 1890 |
| $F | 4068 (2046 in [early RP2A03 revisions](CPU_variants.md#Official_(NTSC))) | 3778 |

The period determines how many CPU cycles happen between shift register clocks. These periods are all even numbers because there are 2 CPU cycles in an APU cycle.

## Length counter load value ($400F, write)

```
7  bit  0
---- ----
LLLL L...
|||| |
++++-+---- Length counter load value
```

Writing to this register also restarts [envelope](APU_Envelope.md).

# Operation

The shift register is 15 bits wide, with bits numbered
14 - 13 - 12 - 11 - 10 - 9 - 8 - 7 - 6 - 5 - 4 - 3 - 2 - 1 - 0

When the timer clocks the shift register, the following actions occur in order:

1. Feedback is calculated as the exclusive-OR of bit 0 and one other bit: bit 6 if [periodic noise mode (M bit)](#Noise_mode_and_period_($400E,_write)) is set, otherwise bit 1.
2. The shift register is shifted right by one bit.
3. Bit 14, the leftmost bit, is set to the feedback calculated earlier.

This results in a pseudo-random bit sequence, 32767 steps long when Mode flag is clear, and randomly 93 or 31 steps long otherwise. (The particular 31- or 93-step sequence depends on where in the 32767-step sequence the shift register was when Mode flag was set).

The [mixer](APU.md#Glossary) receives the current [envelope volume](APU_Envelope.md) except when

* bit 0 of the shift register is set, or
* the [length counter](APU_Length_Counter.md) is zero

Within the mixer, the DMC level has a noticeable effect on the noise's level.

On power-up, the shift register is loaded with the value 1.

In the earliest revisions of the 2A03 CPU, the [periodic noise flag (M bit)](#Noise_mode_and_period_($400E,_write)) was nonexistent: the shift register always used bits 0 and 1 for feedback, and the lowest period (rate $F) lasts 2046 M2 cycles instead of 4068. These CPUs were used in the first batch of Famicom consoles (which were recalled), in Vs. System boards, and in the arcade games that used the 2A03 as a sound coprocessor. [[1]](http://forums.nesdev.org/viewtopic.php?p=58046#p58046), [[2]](https://bsky.app/profile/plgdavid.bsky.social/post/3llhf3ocbx22d)

The 93-step sequence is about a quarter tone (50 cents) sharp relative to A440 tuning. The approximate frequencies and pitches (in [LilyPond's variant of Helmholtz notation](https://en.wikipedia.org/wiki/Helmholtz_pitch_notation#Variations)) are as follows:

<table>
<caption>Pitches of 93-step noise
</caption>
<tr><th></th><th colspan="4">NTSC</th><th colspan="4">PAL</th></tr>
<tr><th>Register value</th><th>Sample rate</th><th>Repeat rate</th><th>MIDI note</th><th>Pitch</th><th>Sample rate</th><th>Repeat rate</th><th>MIDI note</th><th>Pitch</th></tr>
<tr><td>$80</td><td>447443.2 Hz</td><td>4811.2 Hz</td><td>110.41</td><td>d''''' + 41¢</td><td>415651.8 Hz</td><td>4469.4 Hz</td><td>109.13</td><td>c#''''' + 13¢</td></tr>
<tr><td>$81</td><td>223721.6 Hz</td><td>2405.6 Hz</td><td>98.41</td><td>d'''' + 41¢</td><td>207825.9 Hz</td><td>2234.7 Hz</td><td>97.13</td><td>c#'''' + 13¢</td></tr>
<tr><td>$82</td><td>111860.8 Hz</td><td>1202.8 Hz</td><td>86.41</td><td>d''' + 41¢</td><td>118757.6 Hz</td><td>1277.0 Hz</td><td>87.45</td><td>d#''' + 45¢</td></tr>
<tr><td>$83</td><td>55930.4 Hz</td><td>601.4 Hz</td><td>74.41</td><td>d'' + 41¢</td><td>55420.2 Hz</td><td>595.9 Hz</td><td>74.25</td><td>d'' + 25¢</td></tr>
<tr><td>$84</td><td>27965.2 Hz</td><td>300.7 Hz</td><td>62.41</td><td>d' + 41¢</td><td>27710.1 Hz</td><td>298.0 Hz</td><td>62.25</td><td>d' + 25¢</td></tr>
<tr><td>$85</td><td>18643.5 Hz</td><td>200.5 Hz</td><td>55.39</td><td>g + 39¢</td><td>18893.3 Hz</td><td>203.2 Hz</td><td>55.62</td><td>g + 62¢</td></tr>
<tr><td>$86</td><td>13982.6 Hz</td><td>150.4 Hz</td><td>50.41</td><td>d + 41¢</td><td>14089.9 Hz</td><td>151.5 Hz</td><td>50.54</td><td>d + 54¢</td></tr>
<tr><td>$87</td><td>11186.1 Hz</td><td>120.3 Hz</td><td>46.55</td><td>a#, + 55¢</td><td>11233.8 Hz</td><td>120.8 Hz</td><td>46.62</td><td>a#, + 62¢</td></tr>
<tr><td>$88</td><td>8860.3 Hz</td><td>95.3 Hz</td><td>42.51</td><td>f#, + 51¢</td><td>8843.7 Hz</td><td>95.1 Hz</td><td>42.48</td><td>f#, + 48¢</td></tr>
<tr><td>$89</td><td>7046.3 Hz</td><td>75.8 Hz</td><td>38.55</td><td>d, + 55¢</td><td>7044.9 Hz</td><td>75.8 Hz</td><td>38.54</td><td>d, + 54¢</td></tr>
<tr><td>$8A</td><td>4709.9 Hz</td><td>50.6 Hz</td><td>31.57</td><td>g,, + 57¢</td><td>4696.6 Hz</td><td>50.5 Hz</td><td>31.52</td><td>g,, + 52¢</td></tr>
<tr><td>$8B</td><td>3523.2 Hz</td><td>37.9 Hz</td><td>26.55</td><td>d,, + 55¢</td><td>3522.5 Hz</td><td>37.9 Hz</td><td>26.54</td><td>d,, + 54¢</td></tr>
<tr><td>$8C</td><td>2348.8 Hz</td><td>25.3 Hz</td><td>19.53</td><td>g,,, + 53¢</td><td>2348.3 Hz</td><td>25.3 Hz</td><td>19.52</td><td>g,,, + 52¢</td></tr>
<tr><td>$8D</td><td>1761.6 Hz</td><td>18.9 Hz</td><td>14.55</td><td>d,,, + 55¢</td><td>1761.2 Hz</td><td>18.9 Hz</td><td>14.54</td><td>d,,, + 54¢</td></tr>
<tr><td>$8E</td><td>879.9 Hz</td><td>9.5 Hz</td><td>2.53</td><td>d,,,, + 53¢</td><td>879.7 Hz</td><td>9.5 Hz</td><td>2.52</td><td>d,,,, + 52¢</td></tr>
<tr><td>$8F</td><td>440.0 Hz</td><td>4.7 Hz</td><td>-9.47</td><td>d,,,,, + 53¢</td><td>440.1 Hz</td><td>4.7 Hz</td><td>-9.47</td><td>d,,,,, + 53¢</td></tr></table>

<table>
<tr><th colspan="2"><div><a href="https://www.nesdev.org/wiki/Template:APU_nav">V</a> | T</div> <a href="APU.md">APU</a></th></tr>
<tr><td><b>Concepts</b></td><td><a href="APU_registers.md">Registers</a> | <a href="APU_basics.md">Basics</a> | <a href="APU_period_table.md">Period table</a></td></tr>
<tr><td><b>Channels</b></td><td><a href="APU_Pulse.md">Pulse 1 and Pulse 2</a> | <a href="APU_Triangle.md">Triangle</a> | <a>Noise</a> | <a href="APU_DMC.md">DMC</a></td></tr>
<tr><td><b>Components</b></td><td><a href="APU_Envelope.md">Envelope</a> | <a href="APU_Sweep.md">Sweep</a> | <a href="APU_Frame_Counter.md">Frame Counter</a> | <a href="APU_Length_Counter.md">Length Counter</a> | <a href="DMA.md">DMA</a> | <a href="APU_Mixer.md">Mixer</a></td></tr>
<tr><td><b>Expansion audio</b></td><td><a href="https://www.nesdev.org/wiki/FDS_audio">2C33 (Disk System)</a> | <a href="https://www.nesdev.org/wiki/MMC5_audio">MMC5</a> | <a href="https://www.nesdev.org/wiki/VRC6_audio">VRC6</a> | <a href="https://www.nesdev.org/wiki/VRC7_audio">VRC7</a> | <a href="https://www.nesdev.org/wiki/Namco_163_audio">Namco 163</a> | <a href="https://www.nesdev.org/wiki/Sunsoft_5B_audio">Sunsoft 5B</a> | <a href="https://www.nesdev.org/wiki/INES_Mapper_086#Registers">µPD7756</a></td></tr></table>
