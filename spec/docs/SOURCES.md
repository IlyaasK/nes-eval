# Sources and licenses for `spec/docs/`

Everything in this directory was retrieved on 2026-10-06. Only material whose redistribution terms could be
verified on the primary source is included; everything else considered is listed under [Rejected](#rejected).

## Summary

| File or file group | Title | Author(s) | License | Source URL | Retrieval / revision |
|---|---|---|---|---|---|
| `nesdev-wiki/*.md` (69 files) | NESdev Wiki pages (CPU, PPU, APU, DMA, mappers, file formats, input, timing) | NESdev Wiki contributors (per-page history linked in each file header and in the table below) | Public domain, per the wiki's own terms: "Any information posted on this wiki is considered public domain" / "You can use the information from the wiki any way you want" | https://www.nesdev.org/wiki/ (license: https://www.nesdev.org/wiki/Nesdev_wiki:General_disclaimer) | 2026-10-06 via MediaWiki API `action=parse`; per-page revision IDs below |
| `index.md` | Table of contents for this directory | Written for NES Eval | Same terms as the pages it indexes | n/a | 2026-10-06 |
| `SOURCES.md` | This file | Written for NES Eval | n/a | n/a | 2026-10-06 |

No binaries are vendored here, so there are no binary checksums to record.

## NESdev Wiki: license evidence

The wiki has no CC/GFDL license and no `Nesdev wiki:Copyrights` page. The MediaWiki API reports no rights info
(`https://www.nesdev.org/w/api.php?action=query&meta=siteinfo&siprop=rightsinfo` returns `{"url":"","text":""}`),
and `https://www.nesdev.org/wiki/Nesdev_wiki:Copyrights` does not exist. The site footer on every page links
"Disclaimers" to the wiki's own policy page:

* URL: https://www.nesdev.org/wiki/Nesdev_wiki:General_disclaimer
* Revision: 17363 (https://www.nesdev.org/w/index.php?oldid=17363), last edited 2012-12-18 by Tepples, a wiki
  administrator (sysop per `list=allusers&augroup=sysop`). The page was created in June 2009 by user Banshaku.
* Full text of that page at retrieval:

> We cannot guarantee the accuracy of the information available on this wiki.
>
> The general rules for the content is:
> * Any information posted on this wiki is considered public domain
> * You can use the information from the wiki any way you want
>
> For any issues regarding the content, please contact us in the wiki forum of the NESdev BBS, or post the issues on an article's talk page.
>
> The nesdev wiki logo is based on the nesdev BBS one created by Mankeli.

**Required notices:** none. The terms are a public-domain statement with no attribution or share-alike
condition. We attribute anyway: every file starts with a header naming the source page, the exact
revision (`oldid`), the last-edit timestamp, the retrieval date, and a link to the page history listing its
authors ("NESdev Wiki contributors").

**Scope of the grant.** The statement covers text that contributors posted to the wiki. That is why the following
were left out:

* Images. Each uploaded file has its own uploader and source, and many are photos or game screenshots. Their
  terms were not reviewed. Every image is replaced with `[Image omitted]`, and figure captions were removed with
  their images. Small inline icons in tables, such as the manufacturer icons in `Mapper.md`, became their text
  name in brackets (e.g. `[Nintendo]`).
* The `Visual6502wiki/*` pages and `INES Mapper DischDocs` pages. These are copies of third-party documents
  hosted on the wiki (see Rejected below).

**Modifications.** The rendered HTML of each page (`action=parse`, revision as listed) was converted to Markdown.
The conversion removed the site navigation, edit-section links, tables of contents, navboxes, and images. Simple
tables became Markdown pipe tables. Tables with `rowspan`/`colspan` or block content were kept as plain HTML
`<table>`s with all attributes stripped except `rowspan`, `colspan`, `href`, and `id`. Links between included
pages were rewritten to the local `.md` files, following wiki redirects. All other wiki links point to
`https://www.nesdev.org/wiki/…` and do not work offline. Red links (pages that don't exist) became plain
text. The page text was not edited.

### Per-page manifest (NESdev Wiki)

| File | Wiki page | Revision | Last edited |
|---|---|---|---|
| `nesdev-wiki/CPU.md` | [CPU](https://www.nesdev.org/wiki/CPU) | [24046](https://www.nesdev.org/w/index.php?oldid=24046) | 2026-07-17 |
| `nesdev-wiki/CPU_registers.md` | [CPU registers](https://www.nesdev.org/wiki/CPU_registers) | [21223](https://www.nesdev.org/w/index.php?oldid=21223) | 2023-08-09 |
| `nesdev-wiki/CPU_memory_map.md` | [CPU memory map](https://www.nesdev.org/wiki/CPU_memory_map) | [21671](https://www.nesdev.org/w/index.php?oldid=21671) | 2024-03-19 |
| `nesdev-wiki/CPU_addressing_modes.md` | [CPU addressing modes](https://www.nesdev.org/wiki/CPU_addressing_modes) | [1274](https://www.nesdev.org/w/index.php?oldid=1274) | 2018-08-18 |
| `nesdev-wiki/Instruction_reference.md` | [Instruction reference](https://www.nesdev.org/wiki/Instruction_reference) | [24349](https://www.nesdev.org/w/index.php?oldid=24349) | 2026-10-04 |
| `nesdev-wiki/6502_instructions.md` | [6502 instructions](https://www.nesdev.org/wiki/6502_instructions) | [22182](https://www.nesdev.org/w/index.php?oldid=22182) | 2024-11-01 |
| `nesdev-wiki/CPU_unofficial_opcodes.md` | [CPU unofficial opcodes](https://www.nesdev.org/wiki/CPU_unofficial_opcodes) | [23975](https://www.nesdev.org/w/index.php?oldid=23975) | 2026-06-29 |
| `nesdev-wiki/Status_flags.md` | [Status flags](https://www.nesdev.org/wiki/Status_flags) | [22145](https://www.nesdev.org/w/index.php?oldid=22145) | 2024-10-24 |
| `nesdev-wiki/CPU_interrupts.md` | [CPU interrupts](https://www.nesdev.org/wiki/CPU_interrupts) | [21632](https://www.nesdev.org/w/index.php?oldid=21632) | 2024-02-26 |
| `nesdev-wiki/NMI.md` | [NMI](https://www.nesdev.org/wiki/NMI) | [23420](https://www.nesdev.org/w/index.php?oldid=23420) | 2026-01-16 |
| `nesdev-wiki/IRQ.md` | [IRQ](https://www.nesdev.org/wiki/IRQ) | [21856](https://www.nesdev.org/w/index.php?oldid=21856) | 2024-06-04 |
| `nesdev-wiki/CPU_power_up_state.md` | [CPU power up state](https://www.nesdev.org/wiki/CPU_power_up_state) | [22091](https://www.nesdev.org/w/index.php?oldid=22091) | 2024-09-30 |
| `nesdev-wiki/6502_cycle_times.md` | [6502 cycle times](https://www.nesdev.org/wiki/6502_cycle_times) | [23578](https://www.nesdev.org/w/index.php?oldid=23578) | 2026-03-06 |
| `nesdev-wiki/Cycle_reference_chart.md` | [Cycle reference chart](https://www.nesdev.org/wiki/Cycle_reference_chart) | [22030](https://www.nesdev.org/w/index.php?oldid=22030) | 2024-08-26 |
| `nesdev-wiki/Open_bus_behavior.md` | [Open bus behavior](https://www.nesdev.org/wiki/Open_bus_behavior) | [23814](https://www.nesdev.org/w/index.php?oldid=23814) | 2026-05-15 |
| `nesdev-wiki/2A03.md` | [2A03](https://www.nesdev.org/wiki/2A03) | [21333](https://www.nesdev.org/w/index.php?oldid=21333) | 2023-10-08 |
| `nesdev-wiki/CPU_variants.md` | [CPU variants](https://www.nesdev.org/wiki/CPU_variants) | [24230](https://www.nesdev.org/w/index.php?oldid=24230) | 2026-09-08 |
| `nesdev-wiki/Errata.md` | [Errata](https://www.nesdev.org/wiki/Errata) | [24163](https://www.nesdev.org/w/index.php?oldid=24163) | 2026-08-28 |
| `nesdev-wiki/PPU.md` | [PPU](https://www.nesdev.org/wiki/PPU) | [23576](https://www.nesdev.org/w/index.php?oldid=23576) | 2026-03-06 |
| `nesdev-wiki/PPU_registers.md` | [PPU registers](https://www.nesdev.org/wiki/PPU_registers) | [22850](https://www.nesdev.org/w/index.php?oldid=22850) | 2025-07-28 |
| `nesdev-wiki/PPU_memory_map.md` | [PPU memory map](https://www.nesdev.org/wiki/PPU_memory_map) | [22765](https://www.nesdev.org/w/index.php?oldid=22765) | 2025-06-20 |
| `nesdev-wiki/PPU_nametables.md` | [PPU nametables](https://www.nesdev.org/wiki/PPU_nametables) | [22659](https://www.nesdev.org/w/index.php?oldid=22659) | 2025-05-14 |
| `nesdev-wiki/PPU_attribute_tables.md` | [PPU attribute tables](https://www.nesdev.org/wiki/PPU_attribute_tables) | [21522](https://www.nesdev.org/w/index.php?oldid=21522) | 2024-01-16 |
| `nesdev-wiki/PPU_pattern_tables.md` | [PPU pattern tables](https://www.nesdev.org/wiki/PPU_pattern_tables) | [23925](https://www.nesdev.org/w/index.php?oldid=23925) | 2026-06-22 |
| `nesdev-wiki/PPU_palettes.md` | [PPU palettes](https://www.nesdev.org/wiki/PPU_palettes) | [24257](https://www.nesdev.org/w/index.php?oldid=24257) | 2026-09-13 |
| `nesdev-wiki/PPU_OAM.md` | [PPU OAM](https://www.nesdev.org/wiki/PPU_OAM) | [23161](https://www.nesdev.org/w/index.php?oldid=23161) | 2025-10-22 |
| `nesdev-wiki/PPU_rendering.md` | [PPU rendering](https://www.nesdev.org/wiki/PPU_rendering) | [23300](https://www.nesdev.org/w/index.php?oldid=23300) | 2025-12-01 |
| `nesdev-wiki/PPU_scrolling.md` | [PPU scrolling](https://www.nesdev.org/wiki/PPU_scrolling) | [23139](https://www.nesdev.org/w/index.php?oldid=23139) | 2025-10-17 |
| `nesdev-wiki/PPU_sprite_evaluation.md` | [PPU sprite evaluation](https://www.nesdev.org/wiki/PPU_sprite_evaluation) | [22442](https://www.nesdev.org/w/index.php?oldid=22442) | 2025-02-11 |
| `nesdev-wiki/PPU_sprite_priority.md` | [PPU sprite priority](https://www.nesdev.org/wiki/PPU_sprite_priority) | [22861](https://www.nesdev.org/w/index.php?oldid=22861) | 2025-08-03 |
| `nesdev-wiki/PPU_frame_timing.md` | [PPU frame timing](https://www.nesdev.org/wiki/PPU_frame_timing) | [24358](https://www.nesdev.org/w/index.php?oldid=24358) | 2026-10-06 |
| `nesdev-wiki/PPU_power_up_state.md` | [PPU power up state](https://www.nesdev.org/wiki/PPU_power_up_state) | [20192](https://www.nesdev.org/w/index.php?oldid=20192) | 2023-01-07 |
| `nesdev-wiki/Mirroring.md` | [Mirroring](https://www.nesdev.org/wiki/Mirroring) | [24072](https://www.nesdev.org/w/index.php?oldid=24072) | 2026-08-05 |
| `nesdev-wiki/NTSC_video.md` | [NTSC video](https://www.nesdev.org/wiki/NTSC_video) | [24244](https://www.nesdev.org/w/index.php?oldid=24244) | 2026-09-10 |
| `nesdev-wiki/Overscan.md` | [Overscan](https://www.nesdev.org/wiki/Overscan) | [22760](https://www.nesdev.org/w/index.php?oldid=22760) | 2025-06-19 |
| `nesdev-wiki/PPU_variants.md` | [PPU variants](https://www.nesdev.org/wiki/PPU_variants) | [24325](https://www.nesdev.org/w/index.php?oldid=24325) | 2026-09-29 |
| `nesdev-wiki/APU.md` | [APU](https://www.nesdev.org/wiki/APU) | [23811](https://www.nesdev.org/w/index.php?oldid=23811) | 2026-05-15 |
| `nesdev-wiki/APU_basics.md` | [APU basics](https://www.nesdev.org/wiki/APU_basics) | [23439](https://www.nesdev.org/w/index.php?oldid=23439) | 2026-01-19 |
| `nesdev-wiki/APU_registers.md` | [APU registers](https://www.nesdev.org/wiki/APU_registers) | [23438](https://www.nesdev.org/w/index.php?oldid=23438) | 2026-01-19 |
| `nesdev-wiki/APU_Pulse.md` | [APU Pulse](https://www.nesdev.org/wiki/APU_Pulse) | [24334](https://www.nesdev.org/w/index.php?oldid=24334) | 2026-10-01 |
| `nesdev-wiki/APU_Triangle.md` | [APU Triangle](https://www.nesdev.org/wiki/APU_Triangle) | [24352](https://www.nesdev.org/w/index.php?oldid=24352) | 2026-10-04 |
| `nesdev-wiki/APU_Noise.md` | [APU Noise](https://www.nesdev.org/wiki/APU_Noise) | [24354](https://www.nesdev.org/w/index.php?oldid=24354) | 2026-10-05 |
| `nesdev-wiki/APU_DMC.md` | [APU DMC](https://www.nesdev.org/wiki/APU_DMC) | [24164](https://www.nesdev.org/w/index.php?oldid=24164) | 2026-08-29 |
| `nesdev-wiki/APU_Envelope.md` | [APU Envelope](https://www.nesdev.org/wiki/APU_Envelope) | [23446](https://www.nesdev.org/w/index.php?oldid=23446) | 2026-01-19 |
| `nesdev-wiki/APU_Sweep.md` | [APU Sweep](https://www.nesdev.org/wiki/APU_Sweep) | [24338](https://www.nesdev.org/w/index.php?oldid=24338) | 2026-10-02 |
| `nesdev-wiki/APU_Length_Counter.md` | [APU Length Counter](https://www.nesdev.org/wiki/APU_Length_Counter) | [24331](https://www.nesdev.org/w/index.php?oldid=24331) | 2026-09-30 |
| `nesdev-wiki/APU_Frame_Counter.md` | [APU Frame Counter](https://www.nesdev.org/wiki/APU_Frame_Counter) | [23448](https://www.nesdev.org/w/index.php?oldid=23448) | 2026-01-19 |
| `nesdev-wiki/APU_Mixer.md` | [APU Mixer](https://www.nesdev.org/wiki/APU_Mixer) | [23451](https://www.nesdev.org/w/index.php?oldid=23451) | 2026-01-19 |
| `nesdev-wiki/APU_period_table.md` | [APU period table](https://www.nesdev.org/wiki/APU_period_table) | [23440](https://www.nesdev.org/w/index.php?oldid=23440) | 2026-01-19 |
| `nesdev-wiki/DMA.md` | [DMA](https://www.nesdev.org/wiki/DMA) | [23450](https://www.nesdev.org/w/index.php?oldid=23450) | 2026-01-19 |
| `nesdev-wiki/Mapper.md` | [Mapper](https://www.nesdev.org/wiki/Mapper) | [24280](https://www.nesdev.org/w/index.php?oldid=24280) | 2026-09-16 |
| `nesdev-wiki/INES.md` | [INES](https://www.nesdev.org/wiki/INES) | [23194](https://www.nesdev.org/w/index.php?oldid=23194) | 2025-10-27 |
| `nesdev-wiki/NES_2.0.md` | [NES 2.0](https://www.nesdev.org/wiki/NES_2.0) | [24342](https://www.nesdev.org/w/index.php?oldid=24342) | 2026-10-02 |
| `nesdev-wiki/NROM.md` | [NROM](https://www.nesdev.org/wiki/NROM) | [23660](https://www.nesdev.org/w/index.php?oldid=23660) | 2026-03-15 |
| `nesdev-wiki/UxROM.md` | [UxROM](https://www.nesdev.org/wiki/UxROM) | [24147](https://www.nesdev.org/w/index.php?oldid=24147) | 2026-08-26 |
| `nesdev-wiki/CNROM.md` | [CNROM](https://www.nesdev.org/wiki/CNROM) | [24346](https://www.nesdev.org/w/index.php?oldid=24346) | 2026-10-04 |
| `nesdev-wiki/AxROM.md` | [AxROM](https://www.nesdev.org/wiki/AxROM) | [24152](https://www.nesdev.org/w/index.php?oldid=24152) | 2026-08-27 |
| `nesdev-wiki/MMC1.md` | [MMC1](https://www.nesdev.org/wiki/MMC1) | [24091](https://www.nesdev.org/w/index.php?oldid=24091) | 2026-08-16 |
| `nesdev-wiki/MMC3.md` | [MMC3](https://www.nesdev.org/wiki/MMC3) | [24268](https://www.nesdev.org/w/index.php?oldid=24268) | 2026-09-15 |
| `nesdev-wiki/Bus_conflict.md` | [Bus conflict](https://www.nesdev.org/wiki/Bus_conflict) | [24010](https://www.nesdev.org/w/index.php?oldid=24010) | 2026-07-04 |
| `nesdev-wiki/Standard_controller.md` | [Standard controller](https://www.nesdev.org/wiki/Standard_controller) | [23466](https://www.nesdev.org/w/index.php?oldid=23466) | 2026-01-28 |
| `nesdev-wiki/Controller_reading.md` | [Controller reading](https://www.nesdev.org/wiki/Controller_reading) | [23772](https://www.nesdev.org/w/index.php?oldid=23772) | 2026-04-25 |
| `nesdev-wiki/Controller_reading_code.md` | [Controller reading code](https://www.nesdev.org/wiki/Controller_reading_code) | [23961](https://www.nesdev.org/w/index.php?oldid=23961) | 2026-06-27 |
| `nesdev-wiki/Input_devices.md` | [Input devices](https://www.nesdev.org/wiki/Input_devices) | [23692](https://www.nesdev.org/w/index.php?oldid=23692) | 2026-03-28 |
| `nesdev-wiki/Limitations.md` | [Limitations](https://www.nesdev.org/wiki/Limitations) | [22931](https://www.nesdev.org/w/index.php?oldid=22931) | 2025-09-02 |
| `nesdev-wiki/Glossary.md` | [Glossary](https://www.nesdev.org/wiki/Glossary) | [22604](https://www.nesdev.org/w/index.php?oldid=22604) | 2025-04-13 |
| `nesdev-wiki/Myths.md` | [Myths](https://www.nesdev.org/wiki/Myths) | [23006](https://www.nesdev.org/w/index.php?oldid=23006) | 2025-09-25 |
| `nesdev-wiki/Tricky-to-emulate_games.md` | [Tricky-to-emulate games](https://www.nesdev.org/wiki/Tricky-to-emulate_games) | [24361](https://www.nesdev.org/w/index.php?oldid=24361) | 2026-10-06 |
| `nesdev-wiki/Catch-up.md` | [Catch-up](https://www.nesdev.org/wiki/Catch-up) | [18783](https://www.nesdev.org/w/index.php?oldid=18783) | 2021-10-27 |

## Rejected

These sources were considered and not vendored.

| Source | Author(s) | Primary source checked | Reason for rejection |
|---|---|---|---|
| *2A03 technical reference* | Brad Taylor | https://www.nesdev.org/2A03%20technical%20reference.txt | No license or permission statement in the file. The author holds copyright by default. |
| *NTSC 2C02 technical reference* | Brad Taylor | https://www.nesdev.org/2C02%20technical%20reference.TXT | No license or permission statement in the file. |
| *NTSC delta modulation channel documentation* (`dmc.txt`) | Brad Taylor | https://www.nesdev.org/dmc.txt | No license or permission statement in the file. |
| *NES APU Sound Hardware Reference* (`apu_ref.txt`) | blargg (Shay Green) | http://www.slack.net/~ant/nes-emu/apu_ref.txt (copy: https://www.nesdev.org/apu_ref.txt) | The only grant is "Feel free to incorporate this information in references and other documentation." That allows using the *information* but does not clearly allow redistributing the document verbatim, so it was rejected as ambiguous. The wiki APU pages cover the same findings. |
| *Nintendo Entertainment System Documentation* v2.00 (`nestech.txt` in `ndox200.zip`) | Jeremy Chadwick | https://www.nesdev.org/ndox200.zip | The disclaimer says "public-domain information" and in the same paragraph "should not be used for commercial purposes". The terms contradict each other and include a non-commercial restriction. |
| *64doc* (`6502_cpu.txt`) | John West, Marko Mäkelä | https://www.nesdev.org/6502_cpu.txt | Says "See README for copyright notice", but the README is not distributed with it, so the terms cannot be verified. |
| `6502.txt` ("6502 Microprocessor") | unknown | https://www.nesdev.org/6502.txt | Says it is taken from the *Commodore 64 Programmer's Reference* (a Commodore copyright). No license. |
| *6502 Undocumented Opcodes* (`undocumented_opcodes.txt`) | Freddy Offenga | https://www.nesdev.org/undocumented_opcodes.txt | No license or permission statement. |
| *Extra Instructions Of The 65XX Series CPU* (`extra_instructions.txt`) | Adam Vardy | https://www.nesdev.org/extra_instructions.txt | No license or permission statement. |
| `6502bugs.txt` | unknown, posted by Ivo van Poorten | https://www.nesdev.org/6502bugs.txt | Unknown original author, no license. |
| *Nintendo Entertainment System Documentation* (`NESDoc.pdf`) | Patrick Diskin | https://www.nesdev.org/NESDoc.pdf | No license or permission statement. |
| *NMOS 6502 Opcodes* | John Pickens, Bruce Clark, Ed Spittles | http://www.6502.org/tutorials/6502opcodes.html | No license on the page or the site home page (http://www.6502.org/). |
| *Obelisk 6502 Reference* | Andrew John Jacobs | http://www.obelisk.me.uk/6502/ (domain has lapsed and is now a parked page); mirror https://www.nesdev.org/obelisk-6502-guide/ | No license statement on the mirror, and the original site is gone. |
| *6502 Instruction Set* | Norbert Landsteiner (masswerk.at) | https://www.masswerk.at/6502/6502_instruction_set.html | The site timed out on every retrieval attempt (20 s and 60 s), so its terms could not be verified. |
| Disch's mapper docs (*iNES Mappers by Mapper Number*) | Disch | http://www.romhacking.net/documents/362/; copy at https://www.nesdev.org/wiki/INES_Mapper_DischDocs | The document contains no license. It is a copy of a third-party document, so the wiki's public-domain statement cannot relicense it. The wiki's own mapper pages are included instead. |
| NESdev Wiki `Visual6502wiki/*` pages | visual6502.org wiki contributors | https://www.nesdev.org/wiki/Visual6502wiki | The page itself says it is a copy of the Visual6502 project wiki, which went offline in July 2021. The original wiki's terms could not be verified, so the NESdev disclaimer does not clearly cover these pages. |
| NESdev Wiki images (`/w/images/…`) | various uploaders | per-file pages on https://www.nesdev.org/wiki/ | Terms vary per file and were not reviewed. All images are omitted. |
| MOS Technology / Rockwell / Synertek 6502 datasheets and the MCS6500 programming manual | MOS Technology et al. | not searched further | Commercial copyrighted publications with no known redistribution grant. Not pursued because the wiki CPU pages cover the same material. |
