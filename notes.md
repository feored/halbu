### Resources

A list of resources that have helped with reverse engineering the .d2s format.

* http://user.xmission.com/~trevin/DiabloIIv1.09_File_Format.shtm
* https://github.com/oaken-source/pyd2s/blob/master/docs/d2s_save_file_format_1.13d.txt
* https://github.com/WalterCouto/D2CE/blob/main/d2s_File_Format.md
* https://github.com/krisives/d2s-format
* https://github.com/nokka/d2s/blob/master/README.md

All the following observations are tested on patch 2.7/version code 99.

### Character

The name now appears to fill 48 bytes. The names are still limited to 15 graphemes, but not 15 bytes. E.g using japanese kana that each take up 3 bytes, you may fill up 45 bytes of the name section.
You can't mix and match languages, てすと　is valid but てすto is not.

Setting the act/difficulty bytes in the Character section to e.g Hell on a fresh level 1 character won't allow them to enter Hell if they haven't unlocked it. However, setting the act will allow you to access an act you haven't unlocked yet within the same difficulty.


Loading a single player file with "Ladder" bit set to 1 in Character Status does nothing (duh).

The character level shown in the menu preview is the from the attributes section, but in-game it gets overridden by the level in character section.


### Character Menu Appearance

32 bytes starting at offset 136.

**Byte 141:  Weapon**

Some codes gleaned from testing:

* 04: Hand Axe
* 09: Wand
* 0D: Morning Star
* 0F: Flail
* 12: Sabre/Scimitar
* 13: Falchion
* 14: Crystal Sword
* 15: Broad Sword
* 1B: Short Spear/Knife

**Bytes 137..141 and 144..146: Armor**

As far as I can tell, bytes 137..141  the visuals of the body armor, and 144..145 decide the shoulderpads.

Good resource: http://paul.siramy.free.fr/_divers2/Extracting%20Diablo%20II%20Animations.pdf (Page 11)

Bytes 144..145 could be S1 and S2, and 137..141 obviously contain TR and probably RA and LA. More testing needed.

* 02 02 02 02 // 02 02 -- Scale Mail

* 03 03 03 03 // 03 03 -- Full Plate Mail

* 01 02 02 01 // 02 02 -- Studded Leather


### Mercenary

Starts at offset 177

**Mercenary ID**

Offset 179, 4 bytes

Appears to be 0 if you have never hired a merc, otherwise randomly generated.
Can be edited with seemingly no impact.

**Mercenary Name**

Appears unchanged from the list at http://user.xmission.com/~trevin/DiabloIIv1.09_Mercenaries.html

**Mercenary Variant**


| #     | Code      | Difficulty   | Act    | Variant       |
| ----- | ----      | ----------   | ---    | -------       |
| 0     | 00 00     | Normal       | A1     | Fire          |
| 1     | 01 00     | Normal       | A1     | Cold          |
| 2     | 02 00     | Nightmare    | A1     | Fire          |
| 3     | 03 00     | Nightmare    | A1     | Cold          |
| 4     | 04 00     | Hell         | A1     | Fire          |
| 5     | 05 00     | Hell         | A1     | Cold          |
| 6     | 06 00     | Normal       | A2     | Prayer        |
| 7     | 07 00     | Normal       | A2     | Defiance      |
| 8     | 08 00     | Normal       | A2     | Blessed Aim   |
| 9     | 09 00     | Nightmare    | A2     | Thorns        |
| 10    | 0A 00     | Nightmare    | A2     | Holy Freeze   |
| 11    | 0B 00     | Nightmare    | A2     | Might         |
| 12    | 0C 00     | Hell         | A2     | Prayer        |
| 13    | 0D 00     | Hell         | A2     | Defiance      |
| 14    | 0E 00     | Hell         | A2     | Blessed Aim   |
| 15    | 0F 00     | Normal       | A3     | Fire          |    
| 16    | 10 00     | Normal       | A3     | Cold          |    
| 17    | 11 00     | Normal       | A3     | Lightning     |    
| 18    | 12 00     | Nightmare    | A3     | Fire          |    
| 19    | 13 00     | Nightmare    | A3     | Cold          |    
| 20    | 14 00     | Nightmare    | A3     | Lightning     |    
| 21    | 15 00     | Hell         | A3     | Fire          |    
| 22    | 16 00     | Hell         | A3     | Cold          |    
| 23    | 17 00     | Hell         | A3     | Lightning     |
| 24    | 18 00     | Normal       | A5     | Bash          |
| 25    | 19 00     | Normal       | A5     | Bash          |
| 26    | 1A 00     | Nightmare    | A5     | Bash          |
| 27    | 1B 00     | Nightmare    | A5     | Bash          |
| 28    | 1C 00     | Hell         | A5     | Bash          |
| 29    | 1D 00     | Hell         | A5     | Bash          |
| 30    | 1E 00     | Nightmare    | A2     | Prayer        |
| 31    | 1F 00     | Nightmare    | A2     | Defiance      |
| 32    | 20 00     | Nightmare    | A2     | Blessed Aim   |
| 33    | 21 00     | Hell         | A2     | Thorns        |
| 34    | 22 00     | Hell         | A2     | Holy Freeze   |
| 35    | 23 00     | Hell         | A2     | Might         |
| 36    | 24 00     | Normal       | A5     | Frenzy        |
| 37    | 25 00     | Nightmare    | A5     | Frenzy        |
| 38    | 26 00     | Hell         | A5     | Frenzy        |

It appears that the codes have not been changed since: http://user.xmission.com/~trevin/DiabloIIv1.09_Mercenaries.html#code

Instead, the new codes (Nightmare A2 mercs with Prayer/Defiance/Blessed Aim, Hell A2 mercs with Thorns/Holy Freeze/ Might, and Frenzy Barbarians) have been appended to the table.

This also explains why Qual-Kehk usually has more Bash barbs than Frenzy: the two old codes per difficulty still mean bash, whereas there is only one of the new frenzy code per difficulty.

Mercenaries require different amounts of XP to get to a certain level, depending on both their type and the last difficulty beaten by the character they were recruited on.

| Mercenary Type | Variant       | XP Rate (Normal) | XP Rate (Nightmare) | XP Rate (Hell) |
| -------------- | ------------- | ---------------- | ------------------- | -------------- |
| A1             | Fire          |  100             | 110                 |        120     |
| A1             | Cold          |  105             | 115                 |        125     |
| A2             | All           |  110             | 120                 |        130     |
| A3             | Fire          |  110             | 120                 |        130     |
| A3             | Lightning     |  110             | 120                 |        130     |
| A3             | Cold          |  120             | 130                 |        140     |
| A5             | All           |  120             | 130                 |        140     |

The formula to calculate experience based on level and XP Rate is as follows: ```XP Rate * (level + 1) * (level^2)```

Inversely, getting the mercenary level based on XP/rate requires solving the cubic polynomial ```x³ + x² - (Experience/XP Rate) = 0.```
## Statistics

https://d2mods.info/forum/kb/viewarticle?a=448

Open as tab-separated csv.

Incompatible with 1.09 and before.

Saved is col 7
CSvSigned is col 8
CSvBits# is col 9


| Stat                  | ID        | Saved | Signed    | Bits      |
| --------------------- | --------- | ----- | --------- | --------- |
| Strength              | 0         | 1     | 0         | 10        |
| Energy                | 1         | 1     | 0         | 10        |
| Dexterity             | 2         | 1     | 0         | 10        |
| Vitality              | 3         | 1     | 0         | 10        |
| Stat Points Left      | 4         | 1     | 0         | 10        |
| Skill Points Left     | 5         | 1     | 0         | 8         |
| Hit Points (Current)  | 6         | 1     | 0         | 21        |
| Hit Points (Max)      | 7         | 1     | 0         | 21        |
| Mana (Current)        | 8         | 1     | 0         | 21        |
| Mana (Max)            | 9         | 1     | 0         | 21        |
| Stamina (Current)     | 10        | 1     | 0         | 21        |
| Stamina (Max)         | 11        | 1     | 0         | 21        |
| Level                 | 12        | 1     | 0         | 7         |
| Experience            | 13        | 1     | 0         | 32        |
| Gold (Inventory)      | 14        | 1     | 0         | 25        |
| Gold (Stash)          | 15        | 1     | 0         | 25        |


## Quests

Ex: Den of Evil
Quest not started: 0x00 0x00 =>                                 0000 0000   0000 0000
Quest started (Talked to Akara): 0x04 0x00 =>                   0000 0000   0000 0100
Cleared Den of Evil (Return to Akara for reward): 0x1C 0x00 =>  0000 0000   0001 1100
Talked to Akara (Completed quest): 0x01 0x30 =>                 0011 0000   0000 0001
Used skill point: 0x01 0x10 =>                                  0001 0000   0000 0001


Akara reset (offset 82) seems to be set to 2 if unlocked but not used, and to 1 if used.

## Waypoints

A new character will have three waypoints set to true by default: Rogue encampment in normal, nightmare and hell.
Getting to a new act automatically unlocks the town wp.