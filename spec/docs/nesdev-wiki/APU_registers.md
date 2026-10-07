# APU registers

> Source: <https://www.nesdev.org/wiki/APU_registers> — NESdev Wiki, revision [23438](https://www.nesdev.org/w/index.php?oldid=23438) (last edited 2026-01-19T03:43:55Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=APU_registers&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The following memory-mapped registers are used by the [NES APU](APU.md). They are write-only except $4015 which is read/write. Unused registers aren't listed.

*For Famicom expansion audio registers, see also: [Expansion audio](https://www.nesdev.org/wiki/Category:Expansion_audio)*

<table>
<tr><th>Address</th><th><tt>7654.3210</tt></th><th>Function</th></tr>
<tr><td colspan="2"></td><td><b><a href="APU_Pulse.md">Pulse 1 channel</a></b> (write)</td></tr>
<tr><td><b>$4000</b></td><td><tt>DDLC NNNN</tt></td><td><a href="APU_Pulse.md">Duty</a>, <a href="APU_Envelope.md">loop envelope</a>/<a href="APU_Length_Counter.md">disable length counter</a>, <a href="APU_Envelope.md">constant volume</a>, <a href="APU_Envelope.md">envelope period/volume</a></td></tr>
<tr><td><b>$4001</b></td><td><tt>EPPP NSSS</tt></td><td><a href="APU_Sweep.md">Sweep unit</a>: enabled, period, negative, shift count</td></tr>
<tr><td><b>$4002</b></td><td><tt>LLLL LLLL</tt></td><td><a href="APU_Pulse.md">Timer low</a></td></tr>
<tr><td><b>$4003</b></td><td><tt>LLLL LHHH</tt></td><td><a href="APU_Length_Counter.md">Length counter load</a>, <a href="APU_Pulse.md">timer high</a> (also resets <a href="APU_Pulse.md">duty</a> and starts <a href="APU_Envelope.md">envelope</a>)</td></tr>
<tr><td colspan="3"></td></tr>
<tr><td colspan="2"></td><td><b><a href="APU_Pulse.md">Pulse 2 channel</a></b> (write)</td></tr>
<tr><td><b>$4004</b></td><td><tt>DDLC NNNN</tt></td><td><a href="APU_Pulse.md">Duty</a>, <a href="APU_Envelope.md">loop envelope</a>/<a href="APU_Length_Counter.md">disable length counter</a>, <a href="APU_Envelope.md">constant volume</a>, <a href="APU_Envelope.md">envelope period/volume</a></td></tr>
<tr><td><b>$4005</b></td><td><tt>EPPP NSSS</tt></td><td><a href="APU_Sweep.md">Sweep unit</a>: enabled, period, negative, shift count</td></tr>
<tr><td><b>$4006</b></td><td><tt>LLLL LLLL</tt></td><td><a href="APU_Pulse.md">Timer low</a></td></tr>
<tr><td><b>$4007</b></td><td><tt>LLLL LHHH</tt></td><td><a href="APU_Length_Counter.md">Length counter load</a>, <a href="APU_Pulse.md">timer high</a> (also resets <a href="APU_Pulse.md">duty</a> and starts <a href="APU_Envelope.md">envelope</a>)</td></tr>
<tr><td colspan="3"></td></tr>
<tr><td colspan="2"></td><td><b><a href="APU_Triangle.md">Triangle channel</a></b> (write)</td></tr>
<tr><td><b>$4008</b></td><td><tt>CRRR RRRR</tt></td><td><a href="APU_Length_Counter.md">Length counter disable</a>/<a href="APU_Triangle.md">linear counter control</a>, <a href="APU_Triangle.md">linear counter reload value</a></td></tr>
<tr><td><b>$400A</b></td><td><tt>LLLL LLLL</tt></td><td><a href="APU_Triangle.md">Timer low</a></td></tr>
<tr><td><b>$400B</b></td><td><tt>LLLL LHHH</tt></td><td><a href="APU_Length_Counter.md">Length counter load</a>, <a href="APU_Triangle.md">timer high</a> (also reloads <a href="APU_Triangle.md">linear counter</a>)</td></tr>
<tr><td colspan="3"></td></tr>
<tr><td colspan="2"></td><td><b><a href="APU_Noise.md">Noise channel</a></b> (write)</td></tr>
<tr><td><b>$400C</b></td><td><tt>--LC NNNN</tt></td><td><a href="APU_Envelope.md">Loop envelope</a>/<a href="APU_Length_Counter.md">disable length counter</a>, <a href="APU_Envelope.md">constant volume</a>, <a href="APU_Envelope.md">envelope period/volume</a></td></tr>
<tr><td><b>$400E</b></td><td><tt>L--- PPPP</tt></td><td><a href="APU_Noise.md">Loop noise</a>, <a href="APU_Noise.md">noise period</a></td></tr>
<tr><td><b>$400F</b></td><td><tt>LLLL L---</tt></td><td><a href="APU_Length_Counter.md">Length counter load</a> (also starts <a href="APU_Envelope.md">envelope</a>)</td></tr>
<tr><td colspan="3"></td></tr>
<tr><td colspan="2"></td><td><b><a href="APU_DMC.md">DMC channel</a></b> (write)</td></tr>
<tr><td><b>$4010</b></td><td><tt>IL-- FFFF</tt></td><td>IRQ enable, loop sample, frequency index</td></tr>
<tr><td><b>$4011</b></td><td><tt>-DDD DDDD</tt></td><td>Direct load</td></tr>
<tr><td><b>$4012</b></td><td><tt>AAAA AAAA</tt></td><td>Sample address <tt>%11AAAAAA.AA000000</tt></td></tr>
<tr><td><b>$4013</b></td><td><tt>LLLL LLLL</tt></td><td>Sample length <tt>%0000LLLL.LLLL0001</tt></td></tr>
<tr><td colspan="3"></td></tr>
<tr><td><b>$4015</b></td><td><tt>---D NT21</tt></td><td><b><a href="APU.md#Status_($4015)">Control</a></b> (write): <a href="APU_DMC.md">DMC enable</a>, <a href="APU_Length_Counter.md">length counter enables</a>: <a href="APU_Noise.md">noise</a>, <a href="APU_Triangle.md">triangle</a>, <a href="APU_Pulse.md">pulse 2</a>, <a href="APU_Pulse.md">pulse 1</a></td></tr>
<tr><td><b>$4015</b></td><td><tt>IF-D NT21</tt></td><td><b><a href="APU.md#Status_($4015)">Status</a></b> (read): <a href="APU_DMC.md">DMC interrupt</a>, <a href="APU_Frame_Counter.md">frame interrupt</a>, <a href="APU_Length_Counter.md">length counter status</a>: <a href="APU_Noise.md">noise</a>, <a href="APU_Triangle.md">triangle</a>, <a href="APU_Pulse.md">pulse 2</a>, <a href="APU_Pulse.md">pulse 1</a></td></tr>
<tr><td><b>$4017</b></td><td><tt>SD-- ----</tt></td><td><b><a href="APU_Frame_Counter.md">Frame counter</a></b> (write): 5-frame sequence, disable frame interrupt</td></tr></table>

<table>
<tr><th colspan="2"><div><a href="https://www.nesdev.org/wiki/Template:APU_nav">V</a> | T</div> <a href="APU.md">APU</a></th></tr>
<tr><td><b>Concepts</b></td><td><a>Registers</a> | <a href="APU_basics.md">Basics</a> | <a href="APU_period_table.md">Period table</a></td></tr>
<tr><td><b>Channels</b></td><td><a href="APU_Pulse.md">Pulse 1 and Pulse 2</a> | <a href="APU_Triangle.md">Triangle</a> | <a href="APU_Noise.md">Noise</a> | <a href="APU_DMC.md">DMC</a></td></tr>
<tr><td><b>Components</b></td><td><a href="APU_Envelope.md">Envelope</a> | <a href="APU_Sweep.md">Sweep</a> | <a href="APU_Frame_Counter.md">Frame Counter</a> | <a href="APU_Length_Counter.md">Length Counter</a> | <a href="DMA.md">DMA</a> | <a href="APU_Mixer.md">Mixer</a></td></tr>
<tr><td><b>Expansion audio</b></td><td><a href="https://www.nesdev.org/wiki/FDS_audio">2C33 (Disk System)</a> | <a href="https://www.nesdev.org/wiki/MMC5_audio">MMC5</a> | <a href="https://www.nesdev.org/wiki/VRC6_audio">VRC6</a> | <a href="https://www.nesdev.org/wiki/VRC7_audio">VRC7</a> | <a href="https://www.nesdev.org/wiki/Namco_163_audio">Namco 163</a> | <a href="https://www.nesdev.org/wiki/Sunsoft_5B_audio">Sunsoft 5B</a> | <a href="https://www.nesdev.org/wiki/INES_Mapper_086#Registers">µPD7756</a></td></tr></table>
