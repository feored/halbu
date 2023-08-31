### Resources

A list of incredibly useful resources that have helped me understand the .d2s format:

* http://user.xmission.com/~trevin/DiabloIIv1.09_File_Format.shtm
* https://github.com/oaken-source/pyd2s/blob/master/docs/d2s_save_file_format_1.13d.txt
* https://github.com/WalterCouto/D2CE/blob/main/d2s_File_Format.md
* https://github.com/krisives/d2s-format
* https://github.com/nokka/d2s/blob/master/README.md

The following is a list of of observations of my own as I test the editor on D2R patch 2.7, either completing some information here and there or adding things that have been changed since D2R. 

### Character

The name now appears to fill 48 bytes. The names are still limited to 15 graphemes, but not 15 bytes. E.g using japanese kana that each take up 3 bytes, you may fill up 45 bytes of the name section.
You can't mix and match languages, てすと　is valid but てtoす is not.

Setting the act/difficulty bytes in the Character section to e.g Hell on a fresh level 1 character won't allow them to enter Hell if they haven't unlocked it. However, setting the act will allow you to access an act you haven't unlocked yet within the same difficulty.


Loading a single player file with "Ladder" bit set to 1 in Character Status does nothing (duh).

The character level shown in the menu preview is the from the attributes section, but in-game it gets overridden by the level in character section.

Assigned skills have a default value of 0xFF 0xFF 0x00 0x00 before they are set (65535 in lower endian).

### Legacy Character Menu Appearance

32 bytes starting at offset 136.

Default value is 0xFF.

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

As far as I can tell, bytes 137..141 encode the visuals of the body armor, and 144..145 decide the shoulderpads.

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

Appears unchanged from the list at http://user.xmission.com/~trevin/DiabloIIv1.09_Mercenaries.html except for a typo, the first Barbarian name should be Vardakha.

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

The codes have not been changed since 1.09: http://user.xmission.com/~trevin/DiabloIIv1.09_Mercenaries.html#code

Instead, the new codes added with patch 2.4 of D2R (Nightmare A2 mercs with Prayer/Defiance/Blessed Aim, Hell A2 mercs with Thorns/Holy Freeze/ Might, and Frenzy Barbarians) have been appended to the table.

This also explains why Qual-Kehk usually has more Bash barbs than Frenzy: the two old codes per difficulty still mean bash, whereas there is only one of the new frenzy code per difficulty.

Mercenaries require different amounts of XP to get to a certain level, depending on both their type and the last difficulty beaten by the character they were recruited by.

| Mercenary Type | Variant       | XP Rate (Normal) | XP Rate (Nightmare) | XP Rate (Hell) |
| -------------- | ------------- | ---------------- | ------------------- | -------------- |
| A1             | Fire          |  100             | 110                 |        120     |
| A1             | Cold          |  105             | 115                 |        125     |
| A2             | All           |  110             | 120                 |        130     |
| A3             | Fire          |  110             | 120                 |        130     |
| A3             | Lightning     |  110             | 120                 |        130     |
| A3             | Cold          |  120             | 130                 |        140     |
| A5             | All           |  120             | 130                 |        140     |

The formula to calculate experience based on level and XP Rate is as follows: $XP Rate * (Level + 1) * (Level^2)$

Inversely, getting the mercenary level based on current experience and XP Rate requires solving the following cubic polynomial:

$(level + 1)(level^2) = \dfrac{Experience}{XP Rate}$, or using $x$ for $level$: $x³ + x² = \dfrac{Experience}{XP Rate}$

Since x > 0 (the possible values for levels are 1-98), we know that $x^3 < x^3 + x^2 < (x+1)^3$ .

Therefore, $\sqrt[3]{x^3} < \sqrt[3]{x^3 + x^2} < \sqrt[3]{(x + 1)^3}$ or more simply $x < \sqrt[3]{x^3 + x^2} < x + 1$.

Since we know $x^3 + x^2 = \dfrac{Experience}{XP Rate}$, we get the final expression:
$Level < \sqrt[3]{ \dfrac{Experience}{XP Rate}} < Level + 1$

A possible candidate for $x$ is the floor of the cubic root of our experience/xp rate, $s =  \lfloor\sqrt[3]{\dfrac{Experience}{XP Rate}}\rfloor$ . However, it is possible that $s^2 + s^3 > \dfrac{Experience}{XP Rate}$. In that case, we need to take $s - 1$.

The final algorithm is as follows:

```
Let s = floor((experience/xp_rate)**1/3)
if experience/xp_rate < s^3 + s^2:
    return s - 1
else:
    return s
```
#### Example:

A2 Hell mercenary with 99040759 XP. 

```
experience/xp_rate = 99040759/130 = 761851.992308
s = floor(761851.992308^(1/3)) = 91
91^3 + 91^2 = 761852
experience/xp_rate < 761852, therefore level = s - 1 = 90
```
## Attributes

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

The data for every quest is held in 2 bytes. The quests structure contains 8 quests for every act:
one for the introduction to a new act, 6 for every quest (3 in act 4 + padding), and a completion quest.

Some of the flags are constant, others depend on the quest.

Here is the list, from [D2MOO](https://github.com/ThePhrozenKeep/D2MOO/blob/57dcc6ceb493a33dfba82461bd96dd04adb471fe/source/D2CommonDefinitions/include/D2Constants.h#L587):

| Flag                         | Bit |
|  --------------------------- | --- |
| QFLAG_REWARDGRANTED          | 0   |
| QFLAG_REWARDPENDING          | 1   |
| QFLAG_STARTED                | 2   |
| QFLAG_LEAVETOWN              | 3   |
| QFLAG_ENTERAREA              | 4   |
| QFLAG_CUSTOM1                | 5   |
| QFLAG_CUSTOM2                | 6   |
| QFLAG_CUSTOM3                | 7   |
| QFLAG_CUSTOM4                | 8   |
| QFLAG_CUSTOM5                | 9   |
| QFLAG_CUSTOM6                | 10  |
| QFLAG_CUSTOM7                | 11  |
| QFLAG_UPDATEQUESTLOG         | 12  |
| QFLAG_PRIMARYGOALDONE        | 13  |
| QFLAG_COMPLETEDNOW           | 14  |
| QFLAG_COMPLETEDBEFORE        | 15  |

As always, bits should be read right to left.

Example of some stages of Den of Evil:

| Quest Stage                                      | Bytes     | Binary                  | Flags Set                                                      |
| ------------------------------------------------ | --------- | ----------------------- | -------------------------------------------------------------- |
| Quest not started                                | 0x00 0x00 | 0000 0000   0000 0000   | None                                                           |
| Quest started (Talked to Akara)                  | 0x04 0x00 | 0000 0000   0000 0100   | QFLAG_STARTED                                                  |
| Cleared Den of Evil (Return to Akara for reward) | 0x1C 0x00 | 0000 0000   0001 1100   | QFLAG_STARTED QFLAG_LEAVETOWN QFLAG_ENTERAREA                  |
| Talked to Akara (Completed quest)                | 0x01 0x30 | 0011 0000   0000 0001   | QFLAG_REWARDGRANTED QFLAG_UPDATEQUESTLOG QFLAG_PRIMARYGOALDONE |
| Used skill point                                 | 0x01 0x10 | 0001 0000   0000 0001   | QFLAG_REWARDGRANTED QFLAG_UPDATEQUESTLOG                       |

Akara reset (offset 82 out of 96) seems to be set to 2 if unlocked but not used, and to 1 if used.

## Waypoints

A new character will have three waypoints set to true by default: Rogue encampment in normal, nightmare and hell.
Getting to a new act automatically unlocks the town wp.