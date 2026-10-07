# PPU variants

> Source: <https://www.nesdev.org/wiki/PPU_variants> — NESdev Wiki, revision [24325](https://www.nesdev.org/w/index.php?oldid=24325) (last edited 2026-09-29T08:45:18Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=PPU_variants&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

Beyond the well-studied 2C02G, we know of the following PPU revisions, both made by Ricoh and other manufacturers:

## Official

Chips officially licensed by Nintendo for use in official consoles and arcade systems.

### Composite

---

#### NTSC

All official NTSC PPUs expect a 21.477272 MHz master clock.

<table>
<tr><th>Part</th><th>Picture</th><th>First Seen</th><th>Last Seen</th><th>Notes</th></tr>
<tr><td>RP2C02</td><td>[Image omitted] <br/> [Image omitted]</td><td>1983-06 <br/> 3F4 13</td><td>1983-08 <br/> 3H2 10</td><td>Extremely rare. Likely only a few thousand made. <a href="http://web.archive.org/web/20160315221802/kitayama3800.publog.jp/archives/cat_915765.html">http://web.archive.org/web/20160315221802/kitayama3800.publog.jp/archives/cat_915765.html</a> (pictures not archived)]</td></tr>
<tr><td>RP2C02A</td><td>[Image omitted]</td><td>1983-08 <br/> 3H1 43</td><td>1983-10 <br/> 3L1 54</td><td>Sometimes erroneous sprite pixels appear in X=255. Some modern PCBs generate almost exclusively glitchy pattern fetches. <a href="PPU_registers.md#PPUMASK">PPUMASK</a> and <a href="PPU_registers.md#PPUCTRL">PPUCTRL</a> seem to be entirely asynchronous, and writes to <a href="PPU_registers.md#PPUMASK">PPUMASK</a> can disable rendering for one pixel with the commensurate bugs resulting.</td></tr>
<tr><td>RP2C02B</td><td>[Image omitted]</td><td>1983-12 <br/> 3M3 73</td><td>1984-05 <br/> 4E1 54</td><td>Erroneous sprite pixels appear at X=255, just like 2C02A. Production seems to have halted to produce the 2C02C, but 2C02C production stopped and 2C02B production resumed, for unknown reasons.</td></tr>
<tr><td>RP2C02C</td><td>[Image omitted]</td><td>1983-12 <br/> 3M1 10</td><td>1984-02 <br/> 4B4 98</td><td>Eventually stopped being produced in favor of resuming 2C02B production, for unknown reasons.</td></tr>
<tr><td>RC2C02C</td><td>[Image omitted]</td><td colspan="2">1984-01 <br/> 4A3 15</td><td>Comes in a ceramic package.<a href="http://offgao.blog112.fc2.com/blog-entry-28.html">[1]</a> Currently only found inside serviced Famicoms.</td></tr>
<tr><td>RP2C02D</td><td>[Image omitted]</td><td>1984-07 <br/> 4G4 27</td><td>1984-12 <br/> 4M2 29</td><td></td></tr>
<tr><td>RP2C02D-0</td><td>[Image omitted]</td><td>1984-10 <br/> 4K3 63</td><td>1984-12 <br/> 4M2 58</td><td></td></tr>
<tr><td>RP2C02E</td><td>[Image omitted]</td><td>1984-12 <br/> 4M3 14</td><td>1985-10 <br/> 5K4 36</td><td></td></tr>
<tr><td>RP2C02E-0</td><td>[Image omitted]</td><td>1985-03 <br/> 5C5 46</td><td>1987-03 <br/> 7C4 29</td><td>In this and all previous revisions, <a href="PPU_registers.md#OAMDATA">OAMDATA</a> and palette RAM are not readable.
<p><a href="https://forums.nesdev.org/viewtopic.php?p=194740#p194740">Various OAM evaluation bugs</a>
</p></td></tr>
<tr><td>RP2C02G-0</td><td>[Image omitted]</td><td>1987-05 <br/> 7E2 80</td><td>1993-10 <br/> 3KM 1H</td><td>Writes to <a href="PPU_registers.md#OAMADDR">OAMADDR</a> cause corruption of OAM. Leaving OAMADDR at a value of 8 or greater causes OAM corruption when rendering starts. Particularly susceptible to reflections causing OAM corruption, fixed by putting series resistors between the CPU and the cartridge ROM.</td></tr>
<tr><td>RP2C02H-0</td><td>[Image omitted]</td><td>1993-12 <br/> 3MM 40</td><td>1999-05 <br/> 9EM 5B</td><td>Thought have to have fixed some of the glitches in the previous 2C02G-0 revision<sup>(which?)</sup>.</td></tr>
<tr><td>RP2C02H-0 (laser)</td><td>[Image omitted]</td><td>2000-10 <br/> 0KL 40</td><td>2003-01 <br/> 3AL 4B</td><td>Reported (along with PAL PPUs) to have some kind of difference that caused problems with address bus filtering in earlier versions of the Hi-Def NES firmware.</td></tr></table>

#### PAL

All official PAL PPUs expect a 26.601712 MHz master clock.

<table>
<tr><th>Part</th><th>Picture</th><th>First Seen</th><th>Last Seen</th><th>Notes</th></tr>
<tr><td>RP2C07</td><td>[Image omitted]</td><td colspan="2">1985-12 <br/> 5M4 26</td><td>PAL-B PPU. Vblanking is 71 scanlines long. OAM evaluation can never be fully disabled. Red/green color emphasis swapped. <a href="PPU_registers.md#OAMADDR">OAMADDR</a> works correctly, not corrupting OAM contents]</td></tr>
<tr><td>RP2C07-0</td><td>[Image omitted]</td><td>1987-10 <br/> 7K3 27</td><td>1992-01 <br/> 2AM 11</td><td></td></tr>
<tr><td>RP2C07A-0</td><td>[Image omitted]</td><td colspan="2">1992-06 <br/> 2FM 22</td><td>Some subtle differences in PPU that make this work better with Kevtris's HDNES, but otherwise believed identical to 2C07.</td></tr></table>

### RGB

All official RGB PPUs are NTSC and expect a 21.477272 MHz master clock.

---

<table>
<tr><th>Part</th><th>Picture</th><th>First Seen</th><th>Last Seen</th><th>Notes</th></tr>
<tr><td>RC2C03</td><td>[Image omitted]</td><td colspan="2">1983-11 <br/> 3L4 15</td><td>Found in the <a href="https://www.nesdev.org/wiki/Sharp_C1">Sharp C1</a> TV. Suspected to be same core as 2C02 letterless.</td></tr>
<tr><td>RP2C03B</td><td>[Image omitted] <br/> [Image omitted]</td><td>1984-02 <br/> 4B2 36</td><td>1989-03 <br/> 9C4 23</td><td>RGB PPU. Color emphasis bits set the corresponding channel to full brightness. Otherwise believed identical to 2C02B. OAMDATA and PPU palette are not readable.</td></tr>
<tr><td>RC2C03B</td><td>[Image omitted]</td><td>1984-01 <br/> 4A2 14</td><td>1984-01 <br/> 4A4 30</td><td></td></tr>
<tr><td>RP2C03C</td><td>[Image omitted]</td><td colspan="2">1984-03 <br/> 4C2 63</td><td>RGB PPU. Believed to be same core as 2C02C.</td></tr>
<tr><td>RC2C03C</td><td>[Image omitted]</td><td colspan="2">1984-01 <br/> 4A2 10</td><td>Believed identical to RP2C03C.</td></tr>
<tr><td>RP2C04 <br/> 0001</td><td>[Image omitted]</td><td>1984-03 <br/> 4C2 01</td><td>1984-05 <br/> 4E4 35</td><td>RGB PPU with a <a href="PPU_palettes.md#2C04">scrambled palette</a> and some added colors, for copy protection purposes. Equivalent to 2C02D, but keeps 2C03 timing (no skipped dot).</td></tr>
<tr><td>RP2C04 <br/> 0002</td><td>[Image omitted]</td><td>1984-07 <br/> 4G3 15</td><td>1984-10 <br/> 4K2 35</td><td>Scrambles the palette uniquely compared to the previous revision.</td></tr>
<tr><td>RP2C04 <br/> 0003</td><td>[Image omitted]</td><td>1984-10 <br/> 4K5 13</td><td>1984-11 <br/> 4L4 33</td><td>Scrambles the palette uniquely compared to the previous revisions.</td></tr>
<tr><td>RP2C04 <br/> 0004</td><td>[Image omitted]</td><td>1984-11 <br/> 4L3 18</td><td>1984-11 <br/> 4L5 36</td><td>Scrambles the palette uniquely compared to the previous revisions.</td></tr>
<tr><td>RC2C05-01</td><td>[Image omitted]</td><td>1985-06 <br/> 5F5 10</td><td>1985-06 <br/> 5F5 11</td><td><a href="PPU_registers.md#PPUCTRL">PPUCTRL</a> and <a href="PPU_registers.md#PPUMASK">PPUMASK</a> swap locations. Five LSBs of <a href="PPU_registers.md#PPUSTATUS">PPUSTATUS</a> return a constant. Otherwise believed to behave like 2C03.</td></tr>
<tr><td>RC2C05-02</td><td></td><td colspan="2"></td><td>Believed to be the same as 2C05-01, with the PPUSTATUS constant changed.</td></tr>
<tr><td>RC2C05-03</td><td>[Image omitted]</td><td colspan="2">1985-07 <br/> 5G1 12</td><td>Believed to be the same as 2C05-01, with the PPUSTATUS constant changed.</td></tr>
<tr><td>RC2C05-04</td><td>[Image omitted]</td><td colspan="2">1987-03 <br/> 7C3 12</td><td>Believed to be the same as 2C05-01, with the PPUSTATUS constant changed.</td></tr>
<tr><td>RC2C05-99</td><td>[Image omitted]</td><td>1988-11 <br/> 8L4 11</td><td>1988-12 <br/> 8M4 14</td><td>An RGB PPU with RP2C02E behavior: OAM and palettes are unreadable, but OAM corruption added in revision E is present. <a href="PPU_registers.md#PPUSTATUS">PPUSTATUS</a> behavior is normal (the low 5 bits are PPU open bus). Grayscale is normal (palette &amp; $30). Uses 2C02 timing (a dot is skipped every other frame), unlike other RGB PPUs. The palette and emphasis behavior match other RGB PPUs. <a href="PPU_registers.md#PPUCTRL">PPUCTRL</a> and <a href="PPU_registers.md#PPUMASK">PPUMASK</a> aren't swapped. Used in the <a href="https://en.wikipedia.org/wiki/Famicom_Titler">Famicom Titler</a>. (<a href="http://photozou.jp/photo/photo_only/165213/17829839">picture of Titler PCBs</a>)</td></tr></table>

## Unofficial

These chips are found exclusively inside of Famiclone systems, made by multiple companies.

<table>
<tr><th>Part</th><th>Clock</th><th>Video</th><th>Picture</th><th>Notes</th></tr>
<tr><td>UA6528</td><td>21.48<br/>MHz</td><td>NTSC</td><td>[Image omitted] [Image omitted]</td><td>UMC-made clone of 2C02E (or something earlier) <a href="https://forums.nesdev.org/viewtopic.php?&amp;t=20591">[2]</a></td></tr>
<tr><td>UA6528P</td><td>21.49<br/>MHz</td><td>PAL-N</td><td>[Image omitted]</td><td>UMC-made variant for PAL-N (Argentina) <a href="https://forums.nesdev.org/viewtopic.php?f=9&amp;t=13530">[3]</a> System crystal is 21492337.5 Hz (exactly 6×229.2516×15625)</td></tr>
<tr><td>UA6538(P??)</td><td>26.60<br/>MHz</td><td>PAL-B</td><td>[Image omitted]</td><td>UMC-made variant for playing NTSC games in PAL countries. Emits PAL-B video. Vblank interrupt intentionally emitted 50 scanlines later than 2C07. Video DAC is brighter and more saturated <sup>(how so?)</sup>. See also <a href="Cycle_reference_chart.md">Clock rate</a>.</td></tr>
<tr><td>UA6541</td><td>26.60<br/>MHz</td><td>PAL</td><td>[Image omitted]</td><td>UMC-made clone of 2C07 <a href="https://forums.nesdev.org/viewtopic.php?t=17257">[4]</a></td></tr>
<tr><td>UA6548</td><td>21.45<br/>MHz</td><td>PAL-M</td><td>[Image omitted]</td><td>UMC-made variant for PAL-M (Brazil) System crystal is 21453671… Hz (exactly 6×227.25×4500000÷286)</td></tr>
<tr><td>UM6558</td><td>21.31<br/>MHz</td><td>SECAM</td><td>[Image omitted]</td><td>UMC-made variant of UA6538 for SECAM countries. Emits 8-bit "<a href="https://forums.nesdev.org/viewtopic.php?p=194423#p194423">Color Data</a>" digital bus, for conversion into SECAM by UM6559 IC. Color palette <a href="http://www.emu-land.net/forum/index.php/topic,27910.msg1091380.html#msg1091380">noticeably off</a>. System crystal is 21312500 Hz (exactly 4×341×15625). Maybe supports both 50 and 60 Hz operation?</td></tr>
<tr><td>UM6561xx-1</td><td>21.48<br/>MHz</td><td>NTSC</td><td>[Image omitted]</td><td>UMC-made NES-on-a-chip for NTSC. PPU half believed to be mostly identical to UA6528. Revisions "xx" F, AF, BF, CF known.
<p>Emphasis is much stronger than on UA6528 and official PPUs.
</p></td></tr>
<tr><td>UM6561xx-2</td><td>?</td><td>PAL-B</td><td>[Image omitted] [Image omitted]</td><td>UMC-made NES-on-a-chip for PAL-B. PPU half believed to be mostly identical to UA6538. Revisions "xx" F, AF, BF, CF known.
<p>AF revision has graphical and timing glitches in Prince of Persia perhaps caused by sprite 0 hit being missed for reasons not yet understood. F and BF revisions are not affected.
</p><p>Emphasis is much stronger than on UA6538 and official PPUs.
</p></td></tr>
<tr><td>TA-02N</td><td>21.48<br/>MHz</td><td>NTSC</td><td>[Image omitted]
<p>[Image omitted]
</p></td><td>??-made die-mask clone of 2C02G, despite the "6528" label.<a href="https://forums.nesdev.org/viewtopic.php?p=286601#p286601">[5]</a> Chip underside also has two codes of currently unknown purpose.</td></tr>
<tr><td>TA-02NP</td><td>?</td><td>PAL</td><td>[Image omitted] [Image omitted]</td><td>??-made PPU, Dendy compatible (not a 6538 clone). Pins 14-17 are background color in, like normal 2C03<a href="https://forums.nesdev.org/viewtopic.php?p=274318#p274318">[6]</a></td></tr>
<tr><td>TA-02NPB</td><td>?</td><td>Select</td><td>[Image omitted]</td><td>??-made PPU, "NTSC for PAL-B" (NPB) Dendy-compatible. Has a unique video switching capability shared between it and other TA-02NPx chipsets, as well as the WDL chipset.</td></tr>
<tr><td>TA-08</td><td>?</td><td>SECAM</td><td>[Image omitted]</td><td>??-made PPU, Second source alternative to UM6558. Color Data bus bit-reversed?</td></tr>
<tr><td>MG-N-502</td><td>?</td><td>NTSC</td><td>[Image omitted]</td><td></td></tr>
<tr><td>MG-P-502</td><td>26.60<br/>MHz</td><td>PAL-B</td><td>[Image omitted]</td><td>Micro Genius / TXC. Die shot matches UA6538.</td></tr>
<tr><td>1818N</td><td>21.48<br/>MHz</td><td>NTSC</td><td>[Image omitted]</td><td>??-made NES-on-a-chip, NTSC timing.</td></tr>
<tr><td>1818P</td><td>26.60<br/>MHz</td><td>PAL-B</td><td>[Image omitted]</td><td>SiS-made<a href="https://forums.nesdev.org/viewtopic.php?t=24499">[7]</a>] NES-on-a-chip.<a href="https://forums.nesdev.org/viewtopic.php?p=228515#p228515">[8]</a>. Requires external 2KiB RAMs for CPU and PPU. UA6538 timing.</td></tr>
<tr><td>PM02-1</td><td>?</td><td>PAL-M</td><td>[Image omitted]</td><td><a href="https://en.wikipedia.org/wiki/IGB_Eletr%C3%B4nica">Gradiente</a>-made variant for PAL-M (Brazil). <a href="https://forums.nesdev.org/viewtopic.php?p=195175#p195175">[9]</a></td></tr>
<tr><td>VT01</td><td>?</td><td>Select</td><td></td><td>V.R.Tech-made clone of UA6561. Only seen as chip-on-board. Supports composite out; RGB out; or 2bpp STN LCD, either greyscale or <a href="https://www.nesdev.org/wiki/VT01_STN_Palette">red/cyan checkerboard</a>, 240 px wide, 80/120/160/240px tall. The chip was extended significantly in <a href="https://www.nesdev.org/wiki/VTxx#VT02">VT02</a> and newer NOACs.</td></tr>
<tr><td>GS87008</td><td>21.48<br/>MHz</td><td>NTSC</td><td>[Image omitted]</td><td>(Goldstar??)-made NTSC clone <a href="http://gxemu.blog67.fc2.com/blog-entry-363.html">[10]</a>
<p>Found in MicroGenius clone with PAL/NTSC switch.<sup id="cite_ref-1"><a href="#cite_note-1">[1]</a></sup>
</p></td></tr>
<tr><td>GS87008P</td><td>?</td><td>PAL</td><td>[Image omitted]</td><td></td></tr>
<tr><td>KC-6078</td><td>26.60<br/>MHz</td><td>PAL-B</td><td>[Image omitted]</td><td>Found in MT777-DX famiclone, behaves exactly like UA6538</td></tr>
<tr><td>6022</td><td>?</td><td>NTSC</td><td>[Image omitted]</td><td>??-made NTSC clone. Details unknown.</td></tr>
<tr><td>2010</td><td>?</td><td>?</td><td>[Image omitted]</td><td></td></tr>
<tr><td>2A02E</td><td>?</td><td>?</td><td>[Image omitted]</td><td>Dendy Timing. On a large solid-color background (especially yellow, green), every other row has a horizontal stripe.</td></tr>
<tr><td>02</td><td>?</td><td>?</td><td>[Image omitted]</td><td></td></tr>
<tr><td>GT-01</td><td>?</td><td>?</td><td>[Image omitted]</td><td>In PLCC</td></tr>
<tr><td>HA6538</td><td>?</td><td>PAL</td><td>[Image omitted] [Image omitted]</td><td>PAL video output.</td></tr>
<tr><td>SENITON 6538U-8</td><td>?</td><td>PAL</td><td>[Image omitted]</td><td>PAL video output.</td></tr>
<tr><td>SENITON 6538A</td><td>?</td><td>PAL</td><td>[Image omitted]</td><td>PAL video output.</td></tr>
<tr><td>6528 (WDL6528)</td><td>21.48<br/>MHz</td><td>NTSC</td><td>[Image omitted] [Image omitted]</td><td>WDL made chip, often referred to as "WDL6528". NTSC timing. Palette like RP2C02. OAMDATA cannot be read.</td></tr>
<tr><td>8Z02</td><td>21.48<br/>MHz</td><td>NTSC</td><td>[Image omitted]</td><td>Found in Family Game by NTDEC.</td></tr>
<tr><td>TECH28</td><td>?</td><td>PAL</td><td>[Image omitted]</td><td>PAL video output.</td></tr></table>

If you know of other differences or other revisions, please add them!

## See also

* [CPU variants](CPU_variants.md)
* [Clock rate](Cycle_reference_chart.md)
* [NES_2.0#Byte_13_.28Vs._hardware.29](NES_2.0.md#Byte_13_.28Vs._hardware.29)
* <https://forums.nesdev.org/viewtopic.php?p=150127#p150127>

1. [↑](#cite_ref-1) <https://forums.nesdev.org/viewtopic.php?p=241514#p241514>
