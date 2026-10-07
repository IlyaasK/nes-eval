# Input devices

> Source: <https://www.nesdev.org/wiki/Input_devices> — NESdev Wiki, revision [23692](https://www.nesdev.org/w/index.php?oldid=23692) (last edited 2026-03-28T10:27:17Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=Input_devices&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The NES has two [general-purpose controller ports](https://www.nesdev.org/wiki/Controller_port_pinout) on the front of the console, as well as a (rarely used) [48-pin expansion port](https://www.nesdev.org/wiki/Expansion_port#NES) underneath.

The Famicom's standard controllers are hardwired to the front of the unit, and a special [15-pin expansion port](https://www.nesdev.org/wiki/Expansion_port#Famicom) is commonly used for third-party controllers. The AV Famicom, however, features detachable controllers using the same ports as the NES.

The NES and Famicom have a set of I/O ports used for controllers and other peripherals, consisting of the following:

* One output port, 3 bits wide, accessible by writing the bottom 3 bits of $4016.
  * The values latched by $4016/write appear on the [OUT0-OUT2](https://www.nesdev.org/wiki/CPU_pinout) output pins of the 2A03/07, where OUT0 is routed to the controller ports and OUT0-OUT2 to the expansion port on the NES.
* Two input ports, each 5 bits wide, accessible by reading the bottom 5 bits of $4016 and $4017. Reading $4016 and $4017 activates the [/OE1 and /OE2](https://www.nesdev.org/wiki/CPU_pinout) signals, respectively, which are routed to the controller ports and the expansion port.
  * On the NES, only D0, D3, and D4 are connected to both controller ports, while all of D0-D4 are connected to the expansion port.
  * On the original Famicom, the two ports differ: $4016 D0 and D2 and $4017 D0 are permanently connected to both controllers, while $4016 D1 and all of $4017's D0-D4 are connected to the expansion port.
  * On the AV Famicom, only D0 is connected to the controller ports. The expansion port is unchanged.

## Programmer's reference

* [Controller reading](Controller_reading.md)

## Hardware

* [Controller port pinout](https://www.nesdev.org/wiki/Controller_port_pinout)
* Controllers
  * **NES [Standard controller](Standard_controller.md)**
  * [Arkanoid controller](https://www.nesdev.org/wiki/Arkanoid_controller)
  * Bandai Hyper Shot
  * [Coconuts Pachinko](https://www.nesdev.org/wiki/Coconuts_Japan_Pachinko_Controller)
  * Doremikko Keyboard
  * [Exciting Boxing Punching Bag](https://www.nesdev.org/wiki/Exciting_Boxing_Punching_Bag)
  * [Family BASIC Keyboard](https://www.nesdev.org/wiki/Family_BASIC_Keyboard)
  * [Four Score, NES Satellite](https://www.nesdev.org/wiki/Four_player_adapters) 4-player adapters
  * [Hori 4 Players Adapter](https://www.nesdev.org/wiki/Four_player_adapters#Hori_4_Players_Adapter) for Famicom
  * [Jissen Mahjong controller](https://www.nesdev.org/wiki/Jissen_Mahjong_controller)
  * [Konami Hyper Shot](https://www.nesdev.org/wiki/Konami_Hyper_Shot)
  * [Miracle Piano](https://www.nesdev.org/wiki/Miracle_Piano)
  * [Mouse](https://www.nesdev.org/wiki/Mouse) (SNES Mouse, Subor Mouse)
  * [Oeka Kids tablet](https://www.nesdev.org/wiki/Oeka_Kids_tablet)
  * [Partytap](https://www.nesdev.org/wiki/Partytap)
  * Power Glove
  * [Power Pad, Family Trainer, Pokkun Moguraa Tap-tap](https://www.nesdev.org/wiki/Power_Pad) mats
  * RacerMate Bicycle
  * [SNES controller](https://www.nesdev.org/wiki/SNES_controller)
  * Top Rider Bike
  * U-Force
  * [Virtual Boy controller](https://www.nesdev.org/wiki/Virtual_Boy_controller)
  * [Zapper](https://www.nesdev.org/wiki/Zapper) lightgun
* [Infrared controllers](https://www.nesdev.org/wiki/Infrared_controllers)

## Other I/O devices

* [Famicom 3D glasses](https://www.nesdev.org/wiki/Famicom_3D_System)
* [Family BASIC Data Recorder](https://www.nesdev.org/wiki/Family_BASIC_Data_Recorder)
* [R.O.B.](https://www.nesdev.org/wiki/R.O.B.)
* [Battle Box](https://www.nesdev.org/wiki/Battle_Box)
* [Turbo File](https://www.nesdev.org/wiki/Turbo_File)
* Barcode Battler
* [TV-NET Rank 2 controller](https://www.nesdev.org/wiki/TV-NET_Rank_2_controller)
* [FAM-NET Keyboard](https://www.nesdev.org/wiki/FAM-NET_Keyboard)
* [Family Computer Network Adapter](https://www.nesdev.org/wiki/Family_Computer_Network_Adapter)
* [Super Famicom NTT Data Keypad](https://www.nesdev.org/wiki/Super_Famicom_NTT_Data_Keypad)

## Usage of port pins by hardware type

<table>
<tr><td>type</td><th colspan="3">output</th><th colspan="6">Joypad 1</th><th colspan="6">Joypad 2</th><th>audio output</th></tr>
<tr><td>signal</td><th>OUT2</th><th>OUT1</th><th>OUT0</th><th>/OE1</th><th>D4</th><th>D3</th><th>D2</th><th>D1</th><th>D0</th><th>/OE2</th><th>D4</th><th>D3</th><th>D2</th><th>D1</th><th>D0</th><th>AUDIO</th></tr>
<tr><td>access method</td><td colspan="3">write $4016</td><td><sup id="cite_ref-1"><a href="#cite_note-1">[1]</a></sup></td><td colspan="5">read $4016</td><td><sup id="cite_ref-2"><a href="#cite_note-2">[2]</a></sup></td><td colspan="5">read $4017</td><td></td></tr>
<tr><th>available on these ports</th><th colspan="3"></th><th></th><th colspan="5"></th><th></th><th colspan="5"></th><th></th></tr>
<tr><td>Controller port 1 (AV Famicom)</td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td></td><td>D0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td>Controller port 1 (Famicom (internal))</td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td></td><td>D0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Controller_port_pinout">Controller port</a> 1 (NES)</td><td></td><td></td><td>OUT0</td><td>/OE1</td><td>D4</td><td>D3</td><td></td><td></td><td>D0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td>Controller port 2 (AV Famicom)</td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td>/OE2</td><td></td><td></td><td></td><td></td><td>D0</td><td></td></tr>
<tr><td>Controller port 2 (Famicom (internal))</td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td>D2</td><td></td><td></td><td>/OE2</td><td></td><td></td><td></td><td></td><td>D0</td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Controller_port_pinout">Controller port</a> 2 (NES)</td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td>/OE2</td><td>D4</td><td>D3</td><td></td><td></td><td>D0</td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Expansion_port#Famicom">Expansion port</a> (Famicom)</td><td>OUT2</td><td>OUT1</td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td>D1</td><td></td><td>/OE2</td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td>D0</td><td>AUDIO</td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Expansion_port#NES">Expansion port</a> (NES)</td><td>OUT2</td><td>OUT1</td><td>OUT0</td><td>/OE1</td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td>D0</td><td>/OE2</td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td>D0</td><td>AUDIO</td></tr>
<tr><th>used by these devices</th><th colspan="3"></th><th></th><th colspan="5"></th><th></th><th colspan="5"></th><th></th></tr>
<tr><td><a href="Standard_controller.md">Controller</a> (port 1)<sup id="cite_ref-port_3-0"><a href="#cite_note-port-3">[3]</a></sup></td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td></td><td>D0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="Standard_controller.md">Controller</a> (port 2)<sup id="cite_ref-port_3-1"><a href="#cite_note-port-3">[3]</a></sup></td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td>/OE2</td><td></td><td></td><td></td><td></td><td>D0</td><td></td></tr>
<tr><td><a href="Standard_controller.md">Controller</a> (Famicom controller 2)<sup id="cite_ref-4"><a href="#cite_note-4">[4]</a></sup></td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td>D2</td><td></td><td></td><td>/OE2</td><td></td><td></td><td></td><td></td><td>D0</td><td></td></tr>
<tr><td><a href="Standard_controller.md">Controller</a> (expansion port)</td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td>D1</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>AUDIO<sup id="cite_ref-5"><a href="#cite_note-5">[5]</a></sup></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Arkanoid_controller">Arkanoid controller</a> (port 2)<sup id="cite_ref-port_3-2"><a href="#cite_note-port-3">[3]</a></sup></td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td>/OE2</td><td>D4</td><td>D3</td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Arkanoid_controller">Arkanoid controller</a> (expansion port)</td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td>D1</td><td></td><td>/OE2</td><td></td><td></td><td></td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Arkanoid_controller#Arkanoid_II_expansion_port">Arkanoid II controller</a> (2 controllers)</td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td>D1</td><td></td><td>/OE2</td><td>D4</td><td>D3</td><td></td><td>D1</td><td></td><td></td></tr>
<tr><td>Bandai Hyper Shot</td><td>OUT2</td><td>OUT1</td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td>D1</td><td></td><td></td><td>D4</td><td>D3</td><td></td><td></td><td></td><td>AUDIO</td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Exciting_Boxing_Punching_Bag">Exciting Boxing Punching Bag</a></td><td></td><td>OUT1</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/FAM-NET_Keyboard">FAM-NET Keyboard</a></td><td>OUT2</td><td>OUT1</td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Power_Pad#Family_Trainer_Mat">Family Trainer Mat</a></td><td>OUT2</td><td>OUT1</td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Family_BASIC_Keyboard">Family BASIC Keyboard</a></td><td>OUT2</td><td>OUT1</td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Famicom_3D_System">Famicom 3D System</a></td><td></td><td>OUT1</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Famicom_Network_Controller">Famicom Network System controller</a></td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td>D1</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td>Four player adapter (<a href="https://www.nesdev.org/wiki/Four_player_adapters">Four Score</a>)</td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td></td><td>D0</td><td>/OE2</td><td></td><td></td><td></td><td></td><td>D0</td><td></td></tr>
<tr><td>Four player adapter (<a href="https://www.nesdev.org/wiki/Four_player_adapters">Hori 4 Players Adapter</a>)</td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td>D1</td><td></td><td>/OE2</td><td></td><td></td><td></td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Hori_Track">Hori Track</a></td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td>/OE2</td><td></td><td></td><td></td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Jissen_Mahjong_controller">Jissen Mahjong controller</a></td><td>OUT2</td><td>OUT1</td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td>/OE2</td><td></td><td></td><td></td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Konami_Hyper_Shot">Konami Hyper Shot</a></td><td>OUT2</td><td>OUT1</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Oeka_Kids_tablet">Oeka Kids tablet</a></td><td></td><td>OUT1</td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D3</td><td>D2</td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Coconuts_Japan_Pachinko_Controller">Pachinko controller</a></td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td>D1</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Partytap">Party Tap</a></td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td>/OE2</td><td>D4</td><td>D3</td><td>D2</td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Power_Pad#Family_Trainer_Mat">Pokkun Moguraa Tap-tap Mat</a></td><td>OUT2</td><td>OUT1</td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D4</td><td>D3</td><td>D2</td><td>D1</td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Power_Pad">Power Pad</a> (port 2)<sup id="cite_ref-port_3-3"><a href="#cite_note-port-3">[3]</a></sup></td><td></td><td></td><td>OUT0</td><td></td><td></td><td></td><td></td><td></td><td></td><td>/OE2</td><td>D4</td><td>D3</td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Port_test_controller">Port test controller</a></td><td></td><td></td><td>OUT0</td><td>/OE1</td><td>D4</td><td>D3</td><td></td><td></td><td>D0</td><td>/OE2</td><td>D4</td><td>D3</td><td></td><td></td><td>D0</td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/TV-NET_controller">TV-NET controller</a></td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td>D1</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/TV-NET_Rank_2_controller">TV-NET Rank 2 controller</a></td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td>D1</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Zapper">Zapper</a> (port 2)</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D4</td><td>D3</td><td></td><td></td><td></td><td></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Zapper">Zapper</a> (expansion port)</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td>D4</td><td>D3</td><td></td><td></td><td></td><td><sup id="cite_ref-6"><a href="#cite_note-6">[6]</a></sup></td></tr>
<tr><td><a href="https://www.nesdev.org/wiki/Zapper">Zapper</a> (Vs. System) (port 1)<sup id="cite_ref-port_3-4"><a href="#cite_note-port-3">[3]</a></sup></td><td></td><td></td><td>OUT0</td><td>/OE1</td><td></td><td></td><td></td><td></td><td>D0</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr></table>

1. [↑](#cite_ref-1) /OE1 is activated by reading $4016.
2. [↑](#cite_ref-2) /OE2 is activated by reading $4017.
3. ↑ ^[3.0](#cite_ref-port_3-0) ^[3.1](#cite_ref-port_3-1) ^[3.2](#cite_ref-port_3-2) ^[3.3](#cite_ref-port_3-3) ^[3.4](#cite_ref-port_3-4) Controllers using NES ports can be plugged into either port, using that port's /OE and data lines. However, games may expect a controller to only be in a specific port.
4. [↑](#cite_ref-4) The Famicom controller 2 has a microphone that sends audio input over $4016 D2. This is not affected by OUT0 nor /OE2.
5. [↑](#cite_ref-5) A Famicom expansion controller may connect the audio output signal to a headphone jack (for example: IQ502 joypad).
6. [↑](#cite_ref-6) The Casel Zapper plays audio when the trigger is pulled, but this is done entirely by the controller independent of the console's audio out.
