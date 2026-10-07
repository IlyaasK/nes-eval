# CPU variants

> Source: <https://www.nesdev.org/wiki/CPU_variants> — NESdev Wiki, revision [24230](https://www.nesdev.org/w/index.php?oldid=24230) (last edited 2026-09-08T05:31:53Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=CPU_variants&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

Beyond the well-studied 2A03G, we know of the following CPU revisions, both made by Ricoh and other manufacturers:

## Official (NTSC)

All official NTSC CPUs use a ÷12 clock divider.

<table>
<tr><th>Part</th><th>Picture</th><th>First Seen</th><th>Last Seen</th><th>Notes</th></tr>
<tr><td>RP2A03 (ceramic)</td><td>[Image omitted]</td><td colspan="2">1983-06 <br/> 3G1 09</td><td>Likely similar to standard plastic RP2A03.</td></tr>
<tr><td>RP2A03</td><td>[Image omitted] [Image omitted]</td><td>1983-07 <br/> 3G2 ?7</td><td>1984-09 <br/> 4J2 50</td><td>M2 duty cycle is 17/24 instead of 15/24 <a href="https://forums.nesdev.org/viewtopic.php?p=166761#p166761">[1]</a>. Lacks tonal noise mode. Lowest noise period is 2046 instead of 4068. <a href="APU_Frame_Counter.md">APU Frame Counter</a> not restarted on reset. Has broken and disabled <a href="https://www.nesdev.org/wiki/RP2A03_Programmable_Interval_Timer">programmable interval timer</a> on-die. Pin 30 connects to nothing. Other differences?</td></tr>
<tr><td>RP2A03E</td><td>[Image omitted] [Image omitted]</td><td>1984-10 <br/> 4K1 11</td><td>1986-06 <br/> 6F1 23</td><td>Pin 30 is /RDY - combined with internal signals before feeding to internal 6502 +RDY.</td></tr>
<tr><td>RP2A03G</td><td>[Image omitted] [Image omitted]</td><td>1987-04 <br/> 7D3 A0</td><td>1993-11 <br/> 3LM 5B</td><td>Reference model. Pin 30 enables a <a href="https://www.nesdev.org/wiki/CPU_Test_Mode">CPU test mode</a>. Later runs introduced a DMC DMA bug <a href="https://forums.nesdev.org/viewtopic.php?p=275359#p275359">[2]</a>.</td></tr>
<tr><td>RP2A03H</td><td>[Image omitted] [Image omitted]</td><td>1993-12 <br/> 3MM 40</td><td>1999-05 <br/> 9EM 5B</td><td>No known differences from late RP2A03G.</td></tr>
<tr><td>RP2A03H (laser)</td><td>[Image omitted]</td><td>2001-03 <br/> 1CL 42</td><td>2002-11 <br/> 2LL 4A</td><td></td></tr>
<tr><td>RP2A04</td><td>[Image omitted]</td><td colspan="2">1986-03 <br/> 6C2 01</td><td>Not actually a CPU at all, just a jumper in a 40-pin PDIP. Used in place of CPUs in <a href="https://www.nesdev.org/wiki/Vs._System">Vs. System</a> boards (and thus with NTSC timing).</td></tr></table>

## Official (PAL)

All official PAL CPUs use a ÷16 clock divider and they have different [DMC period table](APU_DMC.md) than the NTSC CPUs.

| Part | Picture | First Seen | Last Seen | Notes |
|---|---|---|---|---|
| RP2A07 | [Image omitted] [Image omitted] | 1987-03<br>7C4 39 | 1990-04<br>0DL 2G | Input clock divider is 16. M2 duty cycle is 19/32 [[3]](https://forums.nesdev.org/viewtopic.php?p=166761#p166761). Changes to noise, DPCM, frame timer tables. Fixed DPCM RDY address bus glitches. Pin 30 connects to 6502 /RDY input. |
| RP2A07A | [Image omitted] [Image omitted] | 1991-06<br>1FM 3C | 1992-10<br>2KM 3L | There are no known differences relative to 2A07letterless. |

## Unofficial

There are many unofficial, or "clone" CPUs. Though they generally work quite well for what they are, there are many associated quirks and quality control issues. Keep these items in mind:

* Some clones have reversed duty cycles on the pulse channels, which causes the tone to sound different, but the pitch to remain correct.
* All known clone CPUs use the NTSC [DMC period table](APU_DMC.md), including those intended for PAL systems.
* The clock divider may be 12 (like official NTSC), 16 (like official PAL), or 15 (unique to clones) and this can be tested by dividing the master clock frequency by the observed M2 frequency.
* If you use the incorrect clock divider, the pitch of the sound and the speed of the gameplay will be affected. See [this](https://forums.nesdev.org/viewtopic.php?t=25679) discussion adapting to a different clock divider.

<table>
<tr><th>Part</th><th>Clk<br/>Div</th><th>Picture</th><th>Notes</th></tr>
<tr><td>MG-N-501</td><td>?</td><td>[Image omitted]</td><td></td></tr>
<tr><td><a href="https://forums.nesdev.org/viewtopic.php?p=154574#p154574">MG-P-501</a></td><td>?</td><td>[Image omitted]</td><td>Micro Genius-made clone. Die has the same (UMC) © Ⓜ B6167F marking as a UA6527P.</td></tr>
<tr><td>UA6527</td><td>12</td><td>[Image omitted] [Image omitted]</td><td>UMC-made clone of 2A03G. Has swapped pulse channel duty cycles. Input  clock Divider is 12.</td></tr>
<tr><td rowspan="4">UA6527P</td><td colspan="3">UMC-made clone of 2A03G for compatibility with NTSC software in PAL countries. Different input clock divider. Still has swapped pulse channel duty cycles. Otherwise believed same as 6527.
<p>One revision has (UMC) © Ⓜ B6167F 1989 09 on the die.
</p><p>DMC status bit is cleared 1 APU cycle later than on RP2A03 CPUs. The cause is not known. This changes the timing for <a href="DMA.md#Bugs">DMC DMA implicit-stop glitches</a> (the sample must be started 1 APU cycle earlier to trigger the glitches), and it is suspected that it delays DMC IRQ by 1 APU cycle. Noise channel is slightly louder than others.
</p></td></tr>
<tr><td>16</td><td>[Image omitted]</td><td>Runs hot. Revisions without "-" in the date stamp have a ÷16 CPU divider, like 6540 and 2A07</td></tr>
<tr><td>15</td><td>[Image omitted]</td><td>Runs hot. Revisions with "-" in the date stamp and text on the bottom may have a ÷15 CPU divider.</td></tr>
<tr><td>?</td><td>[Image omitted]</td><td>Runs cooler</td></tr>
<tr><td rowspan="7">UA6527P<br/>(Relabeled)</td><td colspan="3">Note that UA6527P chips are notorious for being other chips that were relabeled.  In some cases, the chip has been sanded and relabeled, but it may still be possible to figure out what it actually is based on markings on the bottom of the chip.  In other cases, the chip was painted and a new label was printed on top of that.  If that's the case, the original label may be revealed by removing the paint.</td></tr>
<tr><td></td><td>[Image omitted]</td><td>Example top of a sanded, painted, and relabeled CPU.  Note the level of freshness on the top side of the chip versus the generally filthy bottom side.  Extrusion marks are not as deep as normal, or may be sanded away entirely.</td></tr>
<tr><td>15</td><td>[Image omitted]</td><td>This CPU has a ÷15 clock divider and correct duty cycles.  These are the same bottom markings as a TA03-NP1.  Pin 30 function is +TST.</td></tr>
<tr><td>15</td><td>[Image omitted]</td><td>This CPU has a ÷15 clock divider and reverse duty cycles.  Pin 30 function is /RDY.</td></tr>
<tr><td>15</td><td>[Image omitted]</td><td>This CPU has a ÷15 clock divider and reverse duty cycles.  Pin 30 function is /RDY.</td></tr>
<tr><td>16</td><td>[Image omitted]</td><td>This CPU has a ÷16 clock divider and reverse duty cycles.</td></tr>
<tr><td>16</td><td>[Image omitted]</td><td>This CPU has a ÷16 clock divider and reverse duty cycles.  Pin 30 function is /RDY.</td></tr>
<tr><td>UA6527PQ</td><td>?</td><td>[Image omitted]</td><td></td></tr>
<tr><td>UA6540</td><td>16</td><td>[Image omitted] [Image omitted]</td><td>UMC-made clone of 2A07 <a href="https://forums.nesdev.org/viewtopic.php?t=17257">[4]</a>. Has swapped pulse duty cycles.
<p>Subsequent research implies this is identical to the early 6527P - NTSC tuning tables, ÷16 CPU divider. <a href="https://forums.nesdev.org/viewtopic.php?p=283514#p283514">[5]</a>
</p></td></tr>
<tr><td>UA6547</td><td>?</td><td>[Image omitted]</td><td>Believed to be a 100% duplicate of UA6527, for use in PAL-M region.</td></tr>
<tr><td>UM6557</td><td>?</td><td>[Image omitted]</td><td>Believed to be a 100% duplicate of UA6527, for use in SECAM regions.</td></tr>
<tr><td>UM6561xx-1</td><td>12</td><td>[Image omitted]</td><td>NES-on-a-chip for NTSC. Revisions "xx" F, AF, BF, CF known.</td></tr>
<tr><td>UM6561xx-2</td><td>?</td><td>[Image omitted] [Image omitted]</td><td>NES-on-a-chip for PAL-B. Revisions "xx" F, AF, BF, CF known.
<p>F and AF revision pulse wave duty cycles match RP2A03, and DMC status bit is cleared 1 APU cycle later than on RP2A03 CPUs.
</p><p>AF revision observed to have incorrect ASR #imm ($4B) behavior, but other stable illegal instructions work properly.
</p></td></tr>
<tr><td>UM6569F-1</td><td>?</td><td>[Image omitted]</td><td>NES-on-a-chip for PAL-N (Argentina). Uses external RAMs. Found on a PCB marked UM6567, implying that one's existence too.</td></tr>
<tr><td>1818N</td><td>?</td><td>[Image omitted]</td><td>??-made NES-on-a-chip, NTSC timing.</td></tr>
<tr><td>T1818P</td><td>?</td><td>[Image omitted]</td><td>??-made NES-on-a-chip[<a href="https://forums.nesdev.org/viewtopic.php?p=228515#p228515">[6]</a>. Requires external 2 KiB RAMs for CPU and PPU. Swapped pulse duty cycles. DMC status bit is cleared 1 APU cycle later than on RP2A03 CPUs.</td></tr>
<tr><td>TA-03N</td><td>12</td><td>[Image omitted]
<p>[Image omitted]
</p></td><td>??-made die-mask clone of 2A03G. Chip underside also has two codes of currently unknown purpose. Pin 30 activates CPU Test Mode like on 2A03G. Clock Divisor is 12. Illegal opcodes are the same. Early 1991 dated chips are reported to have problems with APU DMC playback, but this was corrected in 1992 onward.</td></tr>
<tr><td>TA-03NP</td><td>15<br/>or<br/>12</td><td>[Image omitted]</td><td>??-made clone of 2A03G for NTSC compatibility in PAL countries. Input clock divider is 15?
<p>But not this one, this input clock divider is 12.
</p></td></tr>
<tr><td>TA-03NP1</td><td>15</td><td>[Image omitted]
<p>[Image omitted]
[Image omitted]
</p></td><td>??-made clone of 2A03G for NTSC compatibility in PAL countries. Input clock divider is 15. Fixed DPCM problems? Correct pulse channel duties. Noise channel is slightly louder than others. DMC status bit is cleared 1 APU cycle later than on RP2A03 CPUs.</td></tr>
<tr><td>PM03</td><td>?</td><td>[Image omitted]</td><td><a href="https://en.wikipedia.org/wiki/IGB_Eletr%C3%B4nica">Gradiente</a>-made clone of 2A03G. <a href="https://forums.nesdev.org/viewtopic.php?p=195175#p195175">[7]</a></td></tr>
<tr><td>GS87007</td><td>12</td><td>[Image omitted]</td><td>(Goldstar??)-made clone of 2A03 - has functioning decimal mode? <a href="http://gxemu.blog67.fc2.com/blog-entry-363.html">[8]</a>
<p>Found in MicroGenius clone with PAL/NTSC switch.<sup id="cite_ref-1"><a href="#cite_note-1">[1]</a></sup>
</p></td></tr>
<tr><td>GS87007P</td><td>15</td><td>[Image omitted]</td><td>Contains functional decimal mode, APU pulse duty cycle is swapped. <a href="https://forums.nesdev.org/viewtopic.php?p=307926#p307926">[9]</a></td></tr>
<tr><td>KC-6005</td><td>?</td><td>[Image omitted]</td><td>Found in MT777-DX famiclone, behaves exactly like UA6527P</td></tr>
<tr><td>6005B</td><td>?</td><td>[Image omitted]</td><td></td></tr>
<tr><td>2011</td><td>?</td><td>[Image omitted]</td><td></td></tr>
<tr><td>“2A03E”</td><td>?</td><td>[Image omitted] [Image omitted]</td><td>Both with and without USC insignia</td></tr>
<tr><td>KP2B03E</td><td>?</td><td>[Image omitted]</td><td></td></tr>
<tr><td>6527-21 P03</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are correct. Pin 30 is RDY.</td></tr>
<tr><td>6527</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are correct. Pin 30 is RDY.</td></tr>
<tr><td rowspan="2">6527P</td><td>16</td><td>[Image omitted]</td><td>Clock divider is /16. Pulse channel duty cycles are swapped. Pin 30 is RDY.</td></tr>
<tr><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are correct. Pin 30 is TEST.</td></tr>
<tr><td>HA6527P</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are swapped. Pin 30 is RDY.</td></tr>
<tr><td>SENITON 6527P-SS-P03</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are correct. Pin 30 is TEST.</td></tr>
<tr><td>SENITON 6527UP-8</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are swapped. Pin 30 is RDY.</td></tr>
<tr><td>SENITON 6527AP</td><td>16</td><td>[Image omitted]</td><td>Clock divider is /16. Pulse channel duty cycles are swapped. Pin 30 is RDY.</td></tr>
<tr><td>SL/WH6527AP</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are swapped. Pin 30 is RDY.</td></tr>
<tr><td>SNC6527P</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are correct. Pin 30 is TEST.</td></tr>
<tr><td>XYZ-6783</td><td>?</td><td>[Image omitted]</td><td>Lacks tonal noise mode like original RP2A03, but resets APU Frame Counter on console reset like 2A03E/2A03G. Otherwise behaves like letterless RP2A03.</td></tr>
<tr><td>6538N</td><td>?</td><td>[Image omitted]</td><td>??-made CPU, despite the part number being similar to UMC PPU. Has inverted duty cycles like UA6527. DPCM works.</td></tr>
<tr><td>8Z01N</td><td>12</td><td>[Image omitted]</td><td>Found in Family Game by NTDEC.</td></tr>
<tr><td>TECH 27</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are correct. Pin 30 is TEST.</td></tr>
<tr><td>HITEX-6527P-P03 GX</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are correct. Pin 30 is TEST.</td></tr>
<tr><td>WDL6527P</td><td>15</td><td>[Image omitted]</td><td>Clock divider is /15. Pulse channel duty cycles are correct. Pin 30 is RDY.</td></tr></table>

If you know of other differences or other revisions, please add them!

## See also

* [PPU variants](PPU_variants.md)
* <https://forums.nesdev.org/viewtopic.php?p=45889#p45889>
* <https://forums.nesdev.org/viewtopic.php?t=23916> (More Info on CPU Clones)
* <https://forums.nesdev.org/viewtopic.php?t=23682> (Lots of Images and die-shots)

1. [↑](#cite_ref-1) <https://forums.nesdev.org/viewtopic.php?p=241514#p241514>
