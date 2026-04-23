# v105 Item Bitstream Format — Internal Reference

**Status:** authoritative internal reference for `halbu`'s v105 item handling work
(MASTERPLAN tasks 002–007).
**Scope:** Diablo II: Resurrected, Reign of the Warlock save format
(`FormatId::V105`, save version byte `0x69` / decimal 105).
**Out of scope:** v99 (`0x63`) item format. v99 stays on the existing raw-bytes
placeholder; see `src/items/mod.rs`. `npcs` section. Anything other than the
item bitstream and the four item lists that surround it.

This document is written so that a reader who has not previously seen the
codebase can implement a v105 item parser by walking it top-to-bottom, consulting
only `src/utils.rs` for the bit-order semantics of `read_bits` / `write_bits` and
the Excel files under `assets/excel/v105/` for the data-driven tables that the
bitstream references.

Every documented field is tagged with provenance:

| Tag | Meaning |
|---|---|
| **[Excel]** | Bit width / value derived directly from a column in `assets/excel/v105/*.txt`. |
| **[v99-doc]** | Inherited unchanged from `docs/item-details.html` and `docs/parsing-items.html` (the v99 reference). The v99 ↔ v105 comparison in `docs/v99-v105-details.html` confirms this field is unchanged in v105. |
| **[v99→v105 delta]** | New in v105 relative to v99. Sourced from `docs/v99-v105-details.html`, which in turn is sourced from `dschu012/d2s` PR #86 + a corroborating reviewer comment. |
| **[code]** | Confirmed against existing halbu source code (e.g. `src/items/mod.rs` empty-trailer constants, `src/character/v105.rs` mode marker). |
| **[fixture-unverified]** | Inherited claim that has not been hand-decoded against an `assets/test/v105-real-characters/*.d2s` fixture during the production of this document. Listed in §14 (Open Questions). |

---

## Table of Contents

1. [Overview & Scope](#1-overview--scope)
2. [Bitstream Conventions](#2-bitstream-conventions)
3. [Item-List Framing (the four lists)](#3-item-list-framing-the-four-lists)
4. [Item Header — Simple Bits (every item)](#4-item-header--simple-bits-every-item)
5. [Extended Item Header (non-simple items)](#5-extended-item-header-non-simple-items)
6. [Type-Specific Sub-Blocks (defense, durability, quantity, sockets, set mask)](#6-type-specific-sub-blocks-defense-durability-quantity-sockets-set-mask)
7. [Property List Encoding (Stat-Cost driven)](#7-property-list-encoding-stat-cost-driven)
8. [Special Item-Type Encodings (gold, ear, books, gems/runes/jewels)](#8-special-item-type-encodings-gold-ear-books-gemsrunesjewels)
9. [Location / Equip-Slot Encoding](#9-location--equip-slot-encoding)
10. [Inserted Items (Sockets)](#10-inserted-items-sockets)
11. [Mercenary, Corpse, and Golem Item Lists](#11-mercenary-corpse-and-golem-item-lists)
12. [Shared Stash (`.d2i`) Variant](#12-shared-stash-d2i-variant)
13. [Cross-References to Existing HTML Docs](#13-cross-references-to-existing-html-docs)
14. [Open Questions / Unverified Claims](#14-open-questions--unverified-claims)
15. [Appendix A — Stat-Cost Bit Layout Table](#appendix-a--stat-cost-bit-layout-table)
16. [Appendix B — Per-Fixture Spot-Checks](#appendix-b--per-fixture-spot-checks)

---

## 1. Overview & Scope

A v105 `.d2s` save concludes with an **items tail** that lives at byte offset
`skills_offset + SKILLS_SECTION_LENGTH` (see `src/format/decode.rs`, where
`parsed_save.items = items::parse(&bytes[items_offset..], …)`). That tail is a
**sequence of four item lists**, each with its own framing header, in this
order:

1. Player items — inventory + belt + cube + stash + equipped, all in one list.
2. Corpse items — items lying on the player's last corpse(s), if any.
3. Mercenary items — items equipped on the hired mercenary, if any.
4. Iron Golem item — the single item the Necromancer's Iron Golem is sacrificed
   from, if any.

For RotW expansion saves only, an **additional `lf` trailer** follows the golem
section. The exact byte layout of empty trailers for every (format, expansion,
merc-hire) combination is encoded in `src/items/mod.rs` and reproduced in §3.

Each item inside a list is a **packed bitstream** (no per-item magic header)
that ends on a byte boundary. v105 items are byte-for-byte the same as v99
items with **only two additions** to the bitstream:

- A single bit inserted after the durability fields of extended items
  (§6 / §7 boundary). Purpose unknown; preserve verbatim.
- A 1-bit flag (+ optional 8-bit value) appended after all property lists,
  before the byte-alignment pad. This is the **RotW stash quantity trailer**.
  Applies to **all** items, including simple ones.

Everything else — flags, location, Huffman type code, extended fields, quality
sub-blocks, runeword block, personalized name, defense, durability, sockets,
property lists, sentinel — is identical to v99.

This doc supersedes the misleading parts of `docs/item-details.html`,
`docs/parsing-items.html`, and `docs/moving-items.html` (which all describe v99
exclusively, but are written as if applicable to all D2R) by being explicit
about what is v105 and what is not. See §13 for an explicit cross-reference table.

---

## 2. Bitstream Conventions

These rules apply to every bit-packed field documented in §4–§10. They match
the semantics of `crate::utils::read_bits` / `write_bits` exactly, so a parser
built on those helpers will read what this doc describes without further
adaptation. **[code]**

- **LSB-first within each byte.** The first bit of a multi-bit field occupies
  the least significant bit position of the current byte. After 8 bits are
  consumed, the cursor advances to the next byte. (`read_bits` uses
  `bit::BitIndex::bit_range(low..high)` from low to high inside each byte.)
- **Multi-byte fields are little-endian.** When a field crosses a byte
  boundary, lower-significance bits live in the earlier byte. A 16-bit field
  read out of a fully byte-aligned cursor is identical to
  `u16::from_le_bytes([byte0, byte1])`.
- **No field alignment within an item.** Fields sit back-to-back at the bit
  level. There is no padding between successive fields.
- **Items are byte-aligned at their boundaries.** Each item ends with 0–7 zero
  bits of padding so that the next item (or the next list) starts on a byte
  boundary. The sentinel for "end of properties" (`0x1FF`) is **not** the end
  of the item; the v105 quantity trailer (§7.7) and then alignment follow.
- **Signed values use bias, not a sign bit.** A field whose definition has
  `Save Add = N` is encoded as `raw = value + N` and decoded as
  `value = raw - N`. There is no two's-complement representation. A negative
  decoded value is possible only if `Save Bits` can hold raw values smaller
  than `Save Add`.
- **Cursor positions referenced as "bit X" in this doc** are absolute bit
  offsets from the start of the current item (bit 0 = the LSB of the first byte
  of the item).

---

## 3. Item-List Framing (the four lists)

### 3.1 Order and headers

Within the items tail, sections appear in this fixed order:

| # | Section | Header bytes | Body |
|---|---|---|---|
| 1 | Player items | `4A 4D` (`"JM"`) + `u16` LE count | `count` items, each byte-aligned |
| 2 | Corpse items | `4A 4D` (`"JM"`) + `u16` LE corpse count; then per corpse: 12 bytes (`unk u32 + x u32 + y u32`) + `4A 4D` + `u16` item count + items | per-corpse item list |
| 3 | Mercenary items | `6A 66` (`"jf"`); then `4A 4D` + `u16` count + items. **Only present if a mercenary is hired** (`character.mercenary.is_hired()`) | merc-equipped items |
| 4 | Iron Golem | `6B 66` (`"kf"`) + `u8` `has_golem`; if `has_golem != 0`, exactly one item follows directly (no `JM`, no count) | one item or none |
| 5 (RotW only) | RotW stash trailer | `01 00 6C 66 00 00` (lit. `01 00` + `"lf"` + `00 00`) | unknown payload, see §3.4 |

The corpse-section item list is given only second-hand by the v99 HTML doc
(`docs/item-details.html` table in §2) **[v99-doc, fixture-unverified]**. None
of the v105 fixtures in `assets/test/v105-real-characters/` are believed to
contain a non-empty corpse section; verify when implementing.

### 3.2 Empty trailers (verbatim, from `src/items/mod.rs`) **[code]**

For round-trip parity, every (format, expansion, merc-hire) combination has a
known canonical empty trailer:

```
V105 Classic                     (4 bytes):
  4A 4D 00 00
  // "JM" + count=0  (player)
  // No corpse, no merc, no golem, no lf (classic saves only have player items)

V105 Expansion, no merc          (13 bytes):
  4A 4D 00 00                                  // "JM" + 0  player
  4A 4D 00 00                                  // "JM" + 0  corpse
  6A 66                                        // "jf"      merc header (always present in expansion)
  6B 66 00                                     // "kf" + 0  golem absent

V105 Expansion, with merc        (17 bytes):
  4A 4D 00 00                                  // player
  4A 4D 00 00                                  // corpse
  6A 66 4A 4D 00 00                            // "jf" + "JM" + 0  (empty merc list)
  6B 66 00                                     // "kf" + 0  golem absent

V105 RotW, no merc               (19 bytes):
  4A 4D 00 00                                  // player
  4A 4D 00 00                                  // corpse
  6A 66                                        // "jf"   (no merc list because no merc)
  6B 66 00                                     // "kf" + 0
  01 00                                        // unknown 2-byte prefix
  6C 66                                        // "lf"
  00 00                                        // unknown 2 bytes (empty payload)

V105 RotW, with merc             (23 bytes):
  4A 4D 00 00                                  // player
  4A 4D 00 00                                  // corpse
  6A 66 4A 4D 00 00                            // "jf" + "JM" + 0  (empty merc list)
  6B 66 00                                     // "kf" + 0
  01 00 6C 66 00 00                            // RotW lf trailer
```

> **Note on the merc framing.** The `"jf"` (`6A 66`) bytes are the mercenary-section
> header. In **expansion** saves they are always present, even when no merc is
> hired; only the inner `"JM"` + count list is omitted in that case. This is
> the resolution of the limitation called out in `AGENTS.md` and `NOTES.md`
> (the "blocking compatibility issue" when toggling `mercenary.id` between 0
> and nonzero): toggling adds or removes the inner six bytes
> `4A 4D 00 00` (when the merc has no items) — i.e. the `"JM"` + `u16` count
> wrapper inside the `"jf"` section. With a fully-modeled item list, the
> encoder can re-emit those six bytes (or omit them) based on the new merc
> hire state, so the toggle is no longer blocking.

### 3.3 Iron Golem section: not a `JM` list

The golem section uses a **different framing** from the other three: just a
`u8` flag, and if set, exactly one item follows directly with no count and no
`JM` prefix. This is deliberately not symmetric with the other lists. **[v99-doc]**

### 3.4 RotW `lf` trailer

The v105 RotW empty trailer is `01 00 6C 66 00 00` (six bytes after the
`6B 66 00` golem section). The literal byte sequence is in
`src/items/mod.rs`. **[code]**

The semantics of these six bytes are not yet documented anywhere accessible.
The `"lf"` (`6C 66`) bytes are clearly a section magic by analogy with `"JM"`,
`"jf"`, `"kf"`. The leading `01 00` and trailing `00 00` are unknown.

The most likely interpretation, based on Difference #2 in `v99-v105-details.html`
referring to "RotW's new stash tabs", is that this section frames the
**shared stash inside the character save** (separate from the `.d2i`
shared-stash file in §12) — but this is **fixture-unverified**. See §14 Q1.

When parsing, treat any non-empty `lf` payload as an opaque blob and preserve
it as raw bytes; defer to task 005 (`.d2i` parsing) and a second look at the
v105 fixtures for whether items live here.

---

## 4. Item Header — Simple Bits (every item)

Every item, simple or extended, begins with the same fixed-width 53-bit header,
followed by either an **ear payload** or a **Huffman-encoded type code**.
**[v99-doc]** — confirmed unchanged in v105 by `v99-v105-details.html` §3.2.

### 4.1 Bit-by-bit layout (bits 0–52 of each item)

| Bits | Width | Field | Notes |
|---|---|---|---|
| 0–3 | 4 | reserved | preserved verbatim; observed 0 |
| 4 | 1 | `identified` | 1 = identified |
| 5–10 | 6 | reserved | preserved verbatim |
| 11 | 1 | `socketed` | 1 = item has socket holes (filled or not) |
| 12 | 1 | reserved | |
| 13 | 1 | `new` | 1 = freshly-picked-up, not yet placed |
| 14–15 | 2 | reserved | |
| 16 | 1 | `is_ear` | 1 = ear PvP trophy; takes the ear branch in §8.2 |
| 17 | 1 | `starter_item` | 1 = granted at character creation |
| 18–20 | 3 | reserved | |
| 21 | 1 | `simple_item` | 1 = no extended data block (skip §5–§7) |
| 22 | 1 | `ethereal` | 1 = ethereal |
| 23 | 1 | reserved | observed always 1 (`IFLAG_JUSTSAVED`); preserve |
| 24 | 1 | `personalized` | 1 = renamed via Anya quest |
| 25 | 1 | reserved | possibly `IFLAG_LOWQUALITY` |
| 26 | 1 | `given_runeword` | 1 = runeword active (gates §5.4) |
| 27–31 | 5 | reserved | |
| 32–34 | 3 | `item_version` | observed `101` (binary 5) on D2R items; preserve. **Not** the save-file version. |
| 35–37 | 3 | `location_id` | enum, see §9.1 |
| 38–41 | 4 | `equipped_id` | enum, see §9.2 (only meaningful if `location_id == 1`) |
| 42–45 | 4 | `position_x` | grid column; see §9.4 |
| 46–49 | 4 | `position_y` | grid row; see §9.4 |
| 50–52 | 3 | `alt_position_id` | container enum, see §9.3 |

> **Move-item significance.** The 18 bits at 35–52 are the *only* fields that
> need to change to relocate an item. Everything before and after stays
> bit-identical. See `docs/moving-items.html` for the v99 algorithm — it
> applies unchanged to v105 because these bits are at the same fixed offset.

### 4.2 Branch on `is_ear`

If `is_ear == 1`, parsing skips the type-code Huffman decode and the socket
count, and reads the ear payload described in §8.2 instead. After the ear
payload, jump straight to the v105 quantity trailer (§7.7) and byte alignment.

### 4.3 Item type code (Huffman-encoded, bits 53+)

For non-ear items, the next field is a **Huffman-encoded 4-character type
code** (e.g. `hax `, `rin `, `r01 `). Each character is decoded by reading bits
LSB-first and traversing the Huffman tree (0 = left, 1 = right) until a leaf
is reached. Repeat 4 times. **[v99-doc]** — tree unchanged in v105
(`v99-v105-details.html` §3.2).

The full character table is reproduced in `docs/item-details.html` §10 and
should be embedded in the task-002 codebase verbatim. Bit lengths range from
2 bits (`' '`) to 9 bits (`j`); a typical type code is 17–22 bits. Type codes
are always exactly 4 characters; shorter codes are space-padded *before*
encoding.

The Huffman tree must be embedded as a compile-time constant by task 002.
There is no version branching for the tree itself.

### 4.4 Socket count (1 or 3 bits, depends on item)

After the type code (or after the ear payload if `is_ear`), the parser reads
`nr_of_items_in_sockets` — the number of gems/runes/jewels currently
**inserted** in this item. The width depends on item classification:

| Condition | Width | Reads |
|---|---|---|
| Quest item (item type's category in `itemtypes.txt` includes "Quest") | 2 + 1 bits | 2-bit `quest_difficulty` (stat ID 356) then 1-bit socket count |
| `simple_item == 1` | 1 bit | 1-bit socket count |
| `simple_item == 0` (extended) | 3 bits | 3-bit socket count |

**[v99-doc]** — unchanged in v105.

> The number of *socket holes* (separate from the count of *inserted* items) is
> stored later, as `total_nr_of_sockets` (§6.4), and only for items with the
> `socketed` flag set.

---

## 5. Extended Item Header (non-simple items)

Read this section only if `simple_item == 0` from §4.1 bit 21.

Fields are read in the listed order, back-to-back at the bit level.
**[v99-doc]** — entire section unchanged in v105 (`v99-v105-details.html` §3.3).

### 5.1 Identity, level, quality

| Width | Field | Encoding |
|---|---|---|
| 32 | `id` | random `u32` unique-instance identifier |
| 7 | `level` | item level (ilvl), 1–99 |
| 4 | `quality` | enum, table below |
| 1 | `multiple_pictures` | 1 if a graphic-variant id follows |
| 3 | `picture_id` | only if `multiple_pictures == 1` |
| 1 | `class_specific` | 1 if a class-specific autoaffix id follows |
| 11 | `auto_affix_id` | only if `class_specific == 1`; indexes into `automagic.txt` |

Quality enum:

| Value | Name | Source |
|---|---|---|
| 1 | Low Quality | `lowqualityitems.txt` |
| 2 | Normal | (no sub-block) |
| 3 | Superior | `qualityitems.txt` |
| 4 | Magic | `magicprefix.txt` + `magicsuffix.txt` |
| 5 | Set | `setitems.txt` |
| 6 | Rare | `rareprefix.txt` + `raresuffix.txt` + (magic prefix/suffix tables for affixes) |
| 7 | Unique | `uniqueitems.txt` |
| 8 | Crafted | (same shape as Rare) |

### 5.2 Quality-specific sub-block

Read immediately after `class_specific` / `auto_affix_id`. Fields depend on
`quality`:

| Quality | Bits | Field(s) |
|---|---|---|
| 1 Low | 3 | `low_quality_id` (Crude=0, Cracked=1, Damaged=2, Low Quality=3) |
| 2 Normal | 0 | (none) |
| 3 Superior | 3 | `file_index` (superior bonus type) |
| 4 Magic | 11 + 11 | `magic_prefix` then `magic_suffix` (0 = none on that slot) |
| 5 Set | 12 | `set_id` (index into `setitems.txt`) |
| 6 Rare | 8 + 8 + 6×(1[+11]) | see below |
| 7 Unique | 12 | `unique_id` (index into `uniqueitems.txt`) |
| 8 Crafted | 8 + 8 + 6×(1[+11]) | same shape as Rare |

**Rare/Crafted layout:**

```
8 bits    rare_name_id        // first word of name (RareName table)
8 bits    rare_name_id2       // second word of name
for slot in 0..6:
    1 bit     present_flag
    if present_flag == 1:
        11 bits    affix_id   // alternates prefix/suffix by slot index
```

**Bit-width source:** all widths above are inherited from `docs/item-details.html`
§4.2. **[v99-doc]** — unchanged in v105.

### 5.3 Runeword block (only if `given_runeword == 1`)

```
12 bits    runeword_id        // index into the runes/runewords table
4 bits     padding            // observed always written as decimal 5
```

The 4-bit padding being non-zero is unusual; it is reproduced from
`docs/item-details.html` §4.3 and should be preserved verbatim. The Delirium
runeword has a known quirk where ID 2718 must be remapped to 48 — verify
during task 003 implementation. **[v99-doc, fixture-unverified for v105]**

### 5.4 Personalized name (only if `personalized == 1`)

A null-terminated string of **8-bit characters** (D2R changed from the
classic D2 7-bit-character encoding). Maximum 16 characters including the
null terminator. **[v99-doc]** — unchanged in v105.

```
loop:
    char_byte = read 8 bits
    if char_byte == 0: break
    name.push(char_byte)
```

### 5.5 Tome data (only if item type is `tbk` or `ibk`)

5-bit `tome_data` field that distinguishes the two tome variants (Tome of Town
Portal vs. Tome of Identify) at the binary level. **[v99-doc]**

### 5.6 Realm-data flag (always)

A single bit named `timestamp` / `realm_data_flag` is **always** read for
non-simple items at this point. Its semantics are unclear; preserve verbatim.
**[v99-doc]**

---

## 6. Type-Specific Sub-Blocks (defense, durability, quantity, sockets, set mask)

After the realm-data flag, a sequence of conditional sub-blocks is read,
gated by item-type category and flags. Categories come from `itemtypes.txt`
(see also `armor.txt` / `weapons.txt` / `misc.txt` for the per-item-code
classifications).

### 6.1 Defense rating — armor only

```
11 bits    raw_defense
defense  = raw_defense - 10        // bias from itemstatcost.txt stat 31 (armorclass)
```

Bit width and bias come from **[Excel]** `itemstatcost.txt` row for stat
ID 31 (`armorclass`): `Save Bits = 11`, `Save Add = 10`.

### 6.2 Durability — armor and weapons

```
8 bits    max_durability             // stat 73 (maxdurability)
if max_durability > 0:
    9 bits    current_durability     // stat 72 (durability)
```

Items with `max_durability == 0` are **indestructible base types** (e.g. phase
blades) and the `current_durability` field is omitted. **[Excel]** widths from
`itemstatcost.txt` rows 72 / 73.

### 6.3 v105 unknown bit — extended items only **[v99→v105 delta]**

```
1 bit     v105_unknown_after_durability      // purpose unknown; preserve
```

This bit is **new in v105** (Difference #1 in `docs/v99-v105-details.html` §4).
It is read **only when `simple_item == 0`** (i.e. inside the extended-item
parse path). The PR places the read **after** the durability block, but the
PR's source check does not condition the read on the item being armor or
weapon — see §14 Q2 for the unresolved ambiguity. The safest current
interpretation, matching the PR code, is:

> Always read 1 bit at this position for any extended item, regardless of
> whether the item has durability. Store it, write it back unchanged.

A field such as `v105_unknown_after_durability: bool` on the item model is
sufficient for round-trip fidelity. The bit must **not** be inferred or
recomputed on encode; preserve the parsed value.

### 6.4 Quantity — stackables only

```
9 bits    quantity                  // for arrows, bolts, throwing knives/axes, javelins
```

Item is "stackable" iff its item type in `itemtypes.txt` is in the stackables
set (commonly: `arro`, `bolt`, `tax `, `tkf `, `jav `, plus thrown subtypes).
Confirm exact set against `weapons.txt` and `misc.txt` columns during task 002.
**[v99-doc]** — unchanged in v105 (this is the *internal* per-stack quantity,
distinct from the v105 trailer §7.7 which is a separate field).

### 6.5 Total socket holes — only if `socketed` flag set

```
4 bits    total_nr_of_sockets       // 0–6
```

Distinct from `nr_of_items_in_sockets` (§4.4). **[v99-doc]**

### 6.6 Set bonus mask — only if `quality == 5`

```
5 bits    plist_flag
extra_property_lists = popcount(plist_flag)
```

Each set bit gates one additional set-bonus property list after the main
property list (§7). Number of extra lists = popcount, range 0–5.
**[v99-doc]**

---

## 7. Property List Encoding (Stat-Cost driven)

After all type-specific sub-blocks (§6), a sequence of **property lists**
follows. Each list is a stream of variable-length property records terminated
by a 9-bit sentinel value `0x1FF` (decimal 511). **[v99-doc]** — encoding
unchanged in v105.

### 7.1 Per-property record

```
9 bits    stat_id
if stat_id == 0x1FF: break

prop = ItemStatCost[stat_id]    // from assets/excel/v105/itemstatcost.txt

if prop.save_param_bits > 0:
    param = read prop.save_param_bits bits

raw_value = read prop.save_bits bits
value     = raw_value - prop.save_add
```

Column → field name mapping in `assets/excel/v105/itemstatcost.txt` **[Excel]**:

| Column header | Internal name | Used as |
|---|---|---|
| `Save Bits` | `prop.save_bits` (`sB`) | Width of the value field |
| `Save Add` | `prop.save_add` (`sA`) | Bias subtracted on read, added on write |
| `Save Param Bits` | `prop.save_param_bits` (`sP`) | Width of the optional param field |
| `Encode` | `prop.encode` (`e`) | 0/blank = standard; 2 = chance-to-cast; 3 = charges; 4 = (used by skill-tab; verify) |
| (derived from `Stat` ordering / `descfunc`) | `np` | Number of consecutive sub-stats this id consumes |

The full extracted table is in [Appendix A](#appendix-a--stat-cost-bit-layout-table)
(format and example rows; the complete row dump is left to task 002 codegen).

### 7.2 Multi-property stats (`np > 1`)

When a single `stat_id` covers multiple consecutive stat slots, a single 9-bit
read is followed by `np` value reads (no further `stat_id` reads). Each
sub-entry uses the `Save Bits` / `Save Add` of stat ID `stat_id + i`.

The `np` value is **not** an explicit column in `itemstatcost.txt`; it is
inferred from the layout of the table and from `properties.txt`. Known v105
groupings (carried over from v99 — verify in task 002):

| Lead stat | Stat name | np | Sub-stats |
|---|---|---|---|
| 17 | `item_maxdamage_percent` | 2 | 17 (min%ED), 18 (max%ED) |
| 48 | `firemindam` | 2 | 48 (min), 49 (max) |
| 50 | `lightmindam` | 2 | 50 (min, sB=6), 51 (max, sB=10) |
| 52 | `magicmindam` | 2 | 52 (min), 53 (max) |
| 54 | `coldmindam` | 3 | 54 (min), 55 (max), 56 (length frames) |
| 57 | `poisonmindam` | 3 | 57 (min), 58 (max), 59 (length) |

**[v99-doc]** for this list. The widths in column 4 are sourced from the v105
`itemstatcost.txt` row for the lead stat plus its successors. There are
additional groupings for skill-tab triplets and charges quadruplets that are
expressed via `Encode` rather than `np`; see §7.4 / §7.5.

### 7.3 Standard param-based properties

For stats with `Save Param Bits > 0` and `Encode = 0`, the param is a single
unsigned integer with no internal structure. The most common case is "+N to
single skill (class)" and similar:

```
9 bits    stat_id
sP bits   skill_id (or other context value)
sB bits   raw_value
value = raw_value - sA
```

### 7.4 Skill-tab bonus (`descfunc = 14` style)

For "+N to <Tab> (<Class>)", the param packs both class and tab:

```
9 bits    stat_id
sP bits   param
   tab        = param & 0x7
   class_idx  = (param >> 3) & 0x1FFF
sB bits   value (then subtract sA)
```

**[v99-doc]** — verify on v105 by checking `itemstatcost.txt` `descfunc`
column for the relevant stat IDs (typically 188–194).

### 7.5 `Encode = 2` — chance to cast on event

For stats with `Encode == 2` (e.g. ID 195 `item_skillonattack`,
198 `item_skillonhit`, 201 `item_skillongethit`):

```
9  bits   stat_id
16 bits   param
   skill_level = param & 0x3F        // low 6 bits
   skill_id    = (param >> 6) & 0x3FF // next 10 bits
7  bits   chance (then subtract sA, usually 0)
```

### 7.6 `Encode = 3` — skill charges

For stat ID 204 (`item_charged_skill`):

```
9  bits   stat_id
16 bits   param
   skill_level = param & 0x3F
   skill_id    = (param >> 6) & 0x3FF
16 bits   value
   current_charges = value & 0xFF
   max_charges     = (value >> 8) & 0xFF
```

### 7.7 v105 quantity trailer **[v99→v105 delta]**

After all property lists are written (or after the socket count for simple
items), but **before** byte alignment, v105 reads:

```
1 bit     has_rotw_quantity
if has_rotw_quantity == 1:
    8 bits    rotw_quantity         // RotW stash stack count
```

This is **Difference #2** in `docs/v99-v105-details.html` §5. Critical
properties of this trailer:

- Applies to **all items** — extended *and* simple. The check must not be
  guarded by `simple_item == 0`.
- It is a **separate field** from the 9-bit per-stack `quantity` in §6.4
  (which only applies to stackable weapons).
- The 1-bit flag and conditional 8-bit value are always read in this position;
  the 8-bit value is omitted entirely when the flag is 0 (it does not occupy
  bits in that case).

For round-trip fidelity, store the flag and the conditional value in the item
model. Sensible model fields: `rotw_quantity: Option<u8>` where `None` means
the flag bit was 0 and `Some(n)` means the flag was 1 and the value was `n`.

### 7.8 Property list ordering

Lists appear in this fixed sequence after §6, all using the same per-property
encoding from §7.1:

1. **Main properties** — always present for non-simple items. Terminated by
   `0x1FF`.
2. **Set bonus properties** — `popcount(plist_flag)` lists, gated by §6.6.
   Only for `quality == 5`. Each terminated by its own `0x1FF`.
3. **Runeword properties** — exactly one list, only if `given_runeword == 1`
   from the flags. Terminated by `0x1FF`.

After these, the v105 quantity trailer (§7.7) is read, then the byte alignment
pad is consumed, then any inserted (socketed) items are parsed recursively
(§10).

For simple items, none of the property lists are present; parsing goes
directly from the socket count (§4.4) to the v105 quantity trailer (§7.7) to
alignment.

---

## 8. Special Item-Type Encodings

### 8.1 Gold

The HTML doc does not separately call out gold (`gld `) as a special case at
the bit level. In v105, gold inside an item list is a simple item with item
type `gld `, and the gold quantity is carried by a magic-property record on
the item (stat ID 14 `goldbank` / 15 `goldstash` are the *character*-level
gold counters; per-pile gold uses a different encoding handled inside the
property list). **[fixture-unverified — see §14 Q3]**

### 8.2 Ear (`is_ear == 1`)

When the `is_ear` flag (§4.1 bit 16) is set, the item replaces the type-code
+ socket-count branch with:

```
3 bits    ear_class                  // class index of the slain player
7 bits    ear_level                  // their level at death
loop:                                 // null-terminated 7-bit-per-char name
    char = read 7 bits
    if char == 0: break
    name.push(char)
```

There is no extended data, no socket count, and no inserted items. After the
ear payload, jump straight to the v105 quantity trailer (§7.7) and byte
alignment.

> **Warning.** The ear name uses **7-bit chars** (legacy pre-D2R encoding),
> *not* the 8-bit chars that personalized names use in §5.4. **[v99-doc,
> fixture-unverified for v105 — note that no ears are expected in any of the
> available v105-real-characters fixtures]**

### 8.3 Books / scrolls (`tbk` / `ibk`)

Tome of Town Portal (`tbk`) and Tome of Identify (`ibk`) have a 5-bit
`tome_data` field inserted in the extended-item path immediately after the
personalized-name block (§5.5). All other simple potions, scrolls, and keys
have no special encoding.

### 8.4 Gems / runes / jewels — embedded as inserted items

Gems and runes are themselves **simple items** (§4) with item-type codes from
`gems.txt` and `runes.txt` respectively. Jewels are **non-simple items**
(quality Magic / Rare / Unique) carrying their own property list. When
inserted in another item's sockets, they are written as **complete items**
appended after the parent's byte alignment. See §10.

### 8.5 Ethereal × indestructible interaction

Ethereal items (`ethereal == 1`, §4.1 bit 22) reduce `current_durability`
on read by the in-game ethereal multiplier, but at the **save-file level** the
durability fields are written exactly as for non-ethereal items. No bit-level
adjustment is made. Indestructible items (`max_durability == 0`) skip
`current_durability` regardless of ethereal status.

---

## 9. Location / Equip-Slot Encoding

The four location-related fields at bits 35–52 of every item (§4.1) define
where the item lives. Their values are interrelated and constrained by item
size and container bounds.

### 9.1 `location_id` (3 bits)

| Value | Meaning |
|---|---|
| 0 | Stored in a container (use `alt_position_id`, `position_x`, `position_y`) |
| 1 | Equipped on the body (use `equipped_id`) |
| 2 | In the belt (use `position_x` as belt slot index 0–15; `position_y` ignored) |
| 4 | On cursor / being moved (transient; should never persist on disk) |
| 6 | Socketed inside another item (the parent's inserted-items list owns it) |

### 9.2 `equipped_id` (4 bits, only if `location_id == 1`)

| Value | Slot | `bodylocs.txt` code |
|---|---|---|
| 1 | Helmet | `head` |
| 2 | Amulet | `neck` |
| 3 | Body Armor | `tors` |
| 4 | Right Hand (primary weapon) | `rarm` |
| 5 | Left Hand (primary weapon/shield) | `larm` |
| 6 | Right Ring | `rrin` |
| 7 | Left Ring | `lrin` |
| 8 | Belt | `belt` |
| 9 | Boots | `feet` |
| 10 | Gloves | `glov` |
| 11 | Right Hand (weapon swap II) | — |
| 12 | Left Hand (weapon swap II) | — |

The 12-row `bodylocs.txt` table in `assets/excel/v105/bodylocs.txt` is
reproduced here verbatim **[Excel]**:

```
Body Location   Code
None
Head            head
Neck            neck
Torso           tors
Right Arm       rarm
Left Arm        larm
Right Ring      rrin
Left Ring       lrin
Belt            belt
Feet            feet
Gloves          glov
```

Slots 11–12 (the weapon-swap pair) are not in `bodylocs.txt`; they reuse the
right-arm / left-arm bodyloc codes but with an alternate equip index. **[v99-doc]**

### 9.3 `alt_position_id` (3 bits, only if `location_id == 0`)

| Value | Container |
|---|---|
| 0 | Not in a grid (set when `location_id != 0`) |
| 1 | Inventory |
| 2 | (unknown / unused) |
| 3 | (unknown / unused) |
| 4 | Horadric Cube |
| 5 | Stash (per-character) |

**[v99-doc, fixture-unverified for v105]** — task 003 should verify whether
v105 introduces additional values here for the new RotW shared-stash sub-tabs
or if those use a different mechanism (likely a different item-list section
inside the `lf` trailer; see §3.4 and §14 Q1).

### 9.4 `position_x`, `position_y` (4 + 4 bits)

Grid coordinates within the container identified by `alt_position_id`.
0-indexed. Container dimensions come from `assets/excel/v105/inventory.txt`
**[Excel]** — for D2R, `gridX` × `gridY` is `10 × 4` for the inventory grid
and `10 × 10` for the stash grid (Bank Page 1 differs, see file). The grid
extents constrain the legal range; the parser does not need to validate the
position field, but the move-item API in task 007 must.

For `location_id == 2` (in belt): `position_x` is the belt slot index 0–15
and `position_y` is unused (typically zero). **[v99-doc]**

### 9.5 Mercenary inventory and shared stash

A mercenary's equipped items live in the **mercenary item list** (§3.1 row 3),
not in any dedicated `alt_position_id`. Inside the mercenary list, items use
`location_id = 1` and the merc-relevant `equipped_id` values
(typically Helmet/Body Armor/Right Arm only).

Shared-stash items live in the `.d2i` shared-stash file (§12), or possibly in
the v105 `lf` trailer (§3.4). They use the same item bitstream but with
container coordinates relative to the shared-stash page geometry rather than
the per-character stash.

---

## 10. Inserted Items (Sockets)

If a parsed (parent) item has `nr_of_items_in_sockets > 0` *and*
`simple_item == 0`, then exactly that many **complete child items** follow
immediately after the parent's byte-aligned end. Each child is parsed
recursively from the start of §4.

```
parse parent item up to byte alignment
for i in 0..parent.nr_of_items_in_sockets:
    parse child item        // recursive, full §4–§10 parse
                            // child also byte-aligns at its own end
```

**Constraints on inserted children** (verify during task 003):

- Child `location_id` is 6 ("socketed").
- Children of the same parent appear contiguously in the parent's owning list,
  not in a nested list. Their position in the surrounding list immediately
  follows the parent.
- Children themselves can be simple (gems, runes) or non-simple (jewels);
  the recursion handles both.
- Children with their own sockets are **not expected** but the format does
  not preclude it.

**[v99-doc]** — unchanged in v105. The recursive `nr_of_items_in_sockets`
read for a child is bounded by the child's own `simple_item` flag (1 bit for
simple, 3 bits for extended), not by the parent's.

---

## 11. Mercenary, Corpse, and Golem Item Lists

The framing of these three trailing sections is described in §3.1 and the
empty-trailer byte sequences in §3.2. This section calls out the
relationships between section presence and other save fields.

### 11.1 Mercenary

- The `"jf"` (`6A 66`) section header is **always present** in expansion
  saves, even when no merc is hired. **[code]** confirmed against
  `V105_EMPTY_ITEMS_EXPANSION` and `V105_EMPTY_ITEMS_ROTW` constants in
  `src/items/mod.rs`.
- The inner `"JM"` + `u16` count + items list **is omitted** when the merc is
  not hired. The encoder must check `character.mercenary.is_hired()` (or the
  equivalent `mercenary.id != 0` predicate) and emit/omit those bytes
  accordingly.
- This is the resolution of the `AGENTS.md` "blocking compatibility issue"
  for v105: with a fully-modeled item list, toggling `mercenary.id` between
  0 and nonzero only requires re-emitting (or omitting) the six-byte
  `4A 4D 00 00` empty-list wrapper. The encoder owns this decision; no
  raw-byte preservation of the mercenary-items subsection is needed.
- The mercenary header (14 bytes, see `src/character/v105.rs`
  `RANGE_MERCENARY = 145..159`) is parsed as part of the character section,
  not the items tail. The merc item list comes much later (after attributes
  and skills).

### 11.2 Corpse

- The `"JM"` + `u16` corpse count appears **always**, even when count is 0.
- Per-corpse: a 12-byte block (`unk u32`, `x u32`, `y u32`) followed by an
  inner `"JM"` + `u16` + items list.
- All v105 fixtures in `assets/test/v105-real-characters/` are believed to
  have count = 0; confirm during task 003 implementation.

### 11.3 Iron Golem

- The `"kf"` + `u8` flag is **always present** in expansion saves.
- If the flag is non-zero, exactly **one** complete item follows directly,
  with no count and no `"JM"` prefix.
- This is Necromancer-only in-game; non-Necromancer saves should always have
  the flag = 0.

---

## 12. Shared Stash (`.d2i`) Variant

The `assets/test/v105-real-characters/` directory contains two `.d2i` shared-stash
fixtures that should both parse with the v105 item model:

- `SharedStashSoftCoreV2.d2i`
- `ModernSharedStashSoftCoreV2.d2i`

**No HTML doc and no portion of the existing `src/` codebase describes the
`.d2i` format**, so this section is the most fixture-dependent of the entire
reference. Task 005 (the `.d2i` parsing task) should treat the layout below
as a starting hypothesis to verify, not as confirmed truth.

### 12.1 Hypothesised layout **[fixture-unverified]**

Based on `docs/item-details.html` §2 (table row "Stash page (D2R .d2x)"):

```
0xAA55AA55           4 bytes    page magic, little-endian uint32
hardcore_flag        4 bytes    nonzero = hardcore stash
version              4 bytes    likely matches save format byte (0x69 for v105)
gold                 4 bytes    stash gold counter
size                 4 bytes    page size or item count, semantics TBD
44 bytes             padding / unknown
"JM" + u16 count     ...        item list (same encoding as §3 / §4–§10)
```

This block likely repeats per-page in v2 stashes. The exact relationship
between number of pages, the leading magic, and any inter-page header is
unverified for v105. The two fixtures (`Modern…V2` vs. plain `…V2`) almost
certainly differ in either page count, header version field, or both — see
§14 Q4.

### 12.2 Items inside `.d2i`

Items inside a shared-stash file use the **same bitstream encoding** as items
inside a `.d2s` save (§4–§10), including the v105-specific quantity trailer
in §7.7. There is no separate "stash item" format. **[fixture-unverified]**

### 12.3 No character / quest / waypoint sections

A `.d2i` is a stash-only file; the entire pre-items header that a `.d2s`
carries (signature, version, character section, quests, waypoints, NPC, attributes,
skills) is **absent**. The file begins directly with the stash page header.

---

## 13. Cross-References to Existing HTML Docs

| HTML doc | Section/Claim | Status in v105 | Corrected/Authoritative statement |
|---|---|---|---|
| `item-details.html` | Title: "Save file version: 99 (0x63)" | Outdated for v105 | This doc covers v105 (0x69). The item bitstream is **mostly** identical (see `v99-v105-details.html` §3) but two bit-level additions apply (§6.3 and §7.7 here). |
| `item-details.html` | §2 table row: per-item `JM` removed in D2R | **Correct** for both v99 and v105 | Reaffirmed; the per-item `JM` is gone in all D2R versions (v97+). The four item-list wrappers still use `JM`. (Cf. `NOTES.md` "The JM header at the top of every item has been removed in D2R" — same statement.) |
| `item-details.html` | §3.1 Flags layout (32 bits) | **Correct** | Unchanged in v105. |
| `item-details.html` | §3.2 Item version: 3 bits at 32–34, value `101` | **Correct** | Unchanged in v105. |
| `item-details.html` | §3.3 Location: 18 bits at 35–52 | **Correct** | Unchanged in v105. The fixed-offset property is the basis for the move-item algorithm. |
| `item-details.html` | §3.4 Huffman tree (full table in §10) | **Correct** | Tree unchanged in v105 (per `v99-v105-details.html` §3.2; PR #86 does not modify it). Verify during task 003 by decoding type codes from a v105 fixture. |
| `item-details.html` | §4 Extended data block | **Correct** | All sub-blocks (id, level, quality, quality-specific, runeword, personalized, tome, realm flag) are unchanged in v105. |
| `item-details.html` | §4.4 Personalized name = 8-bit chars | **Correct** | Unchanged in v105. (Note: ear names in §8.2 still use 7-bit chars.) |
| `item-details.html` | §4.6 Defense (11b), Durability (8b/9b), Quantity (9b) | **Correct** | Bit widths unchanged. **However**, in v105 a single bit is inserted between durability and quantity (§6.3 here). The HTML doc does not mention this bit. |
| `item-details.html` | §5 Magic properties (9-bit stat_id + sB/sP, terminator 0x1FF) | **Correct** | Encoding unchanged. The values in `itemstatcost.txt` (sB, sA, sP) for v105 may differ for some stats; verify during task 002 by diffing v99 vs. v105 `itemstatcost.txt`. |
| `item-details.html` | §11 Stat IDs Quick Reference | **Likely correct** but version-tagged as "99 constant data" | Re-extract from `assets/excel/v105/itemstatcost.txt` for v105 ground truth. The HTML reference is convenient but not authoritative for v105 numeric widths. |
| `item-details.html` | §7 Byte alignment | **Correct** but **incomplete** for v105 | The HTML places alignment immediately after the property lists. In v105, the quantity trailer (§7.7 here) is read **between** the last `0x1FF` and the alignment pad. |
| `parsing-items.html` | All sections | Same status as `item-details.html` (it is a parallel walkthrough of the v99 format) | Apply the same v105 deltas: §6.3 and §7.7 here. |
| `moving-items.html` | The "patch 18 bits at fixed offset" technique | **Correct** for v105 | The technique works identically for v105 because bits 35–52 are at the same fixed offset and v105 adds bits only **after** them. The boundary-finding caveats in `moving-items.html` §3 (no length prefix, must parse whole item) apply equally. |
| `v99-v105-details.html` | Entire document | **Authoritative** for the v99↔v105 deltas | This doc is the source of truth for §6.3 and §7.7. Confidence ratings ("Confirmed" / "Likely" / "Unknown") in that doc are inherited verbatim into §14 here. |
| `v99-v105-details.html` | §2.2 Empty trailer (`lf` section) | **Correct** | The 19/23-byte empty trailers are exactly what `src/items/mod.rs` emits. |
| `v99-v105-details.html` | §8 Q5: Huffman tree changed in v105? | **Likely no** | Inherit as a verification task for task 003. |
| `NOTES.md` | "The JM header at the top of every item has been removed in D2R" | **Correct** | Reaffirmed — applies to v97 onward, including v105. |
| `NOTES.md` | "Halbu does not currently rewrite the jf mercenary item subsection" | **Resolved** by this work | With the items section fully modeled, the encoder can re-emit the `jf` section's contents based on the current merc hire state. See §11.1. |

---

## 14. Open Questions / Unverified Claims

These are claims that this document inherits without independent fixture
verification, or that need an extra pass during task 003 (parser implementation).

| # | Question | Confidence | Where it matters | Resolution path |
|---|---|---|---|---|
| Q1 | Does the v105 RotW `lf` (`6C 66`) trailer carry the new RotW shared-stash tabs as item lists, or is it a fixed-shape opaque payload? | Unknown | §3.4, §9.5 | Compare an RotW fixture with items in the new stash tabs against an empty-RotW save. If the bytes after `6C 66` change shape with stashed items, those items live there and §9.3's `alt_position_id` table needs an extra value (or a new container code). |
| Q2 | Does the v105 unknown bit in §6.3 fire for **all** extended items, or only those with durability (armor/weapons)? | Likely all extended items, per the PR code's placement | §6.3 | Decode a v105 rare ring (no durability) and check whether 1 bit is consumed at the §6.3 position. The PR places the `SkipBits(1)` after the durability block but does not condition it on durability presence; the v99→v105 doc highlights this ambiguity. |
| Q3 | How is gold (`gld `) stack quantity encoded inside an item list? Is it a property record, the v105 quantity trailer (§7.7), or the type-internal stackable-quantity field (§6.4)? | Unknown for v105 | §8.1 | Decode the gold pile carried in a fixture (e.g. `Locker.d2s`) and compare its item bytes against a known gold-pile encoding. The 1-bit + 8-bit RotW trailer (§7.7) max value is 255 — too small for in-game gold piles, so per-pile gold likely uses §6.4 or a property record. |
| Q4 | What are the bit-level differences between `SharedStashSoftCoreV2.d2i` and `ModernSharedStashSoftCoreV2.d2i`? | Unknown | §12 | Hex-dump the two fixture headers; diff. Most likely a `version` field in the page header differs. |
| Q5 | Are the v105 `itemstatcost.txt` `Save Bits` / `Save Add` / `Save Param Bits` values **identical** to v99 for every shared stat? | Likely identical | §7, Appendix A | Diff `assets/excel/v105/itemstatcost.txt` against the v99 drop. If any width differs for a stat that appears in v105 fixture items, every property of that stat would be misread; high-impact. |
| Q6 | Are there **new** stat IDs in v105 (e.g. for Warlock skills)? | Likely yes | §7, Appendix A | Diff `itemstatcost.txt` row count and content; v105 has 369 rows in `assets/excel/v105/itemstatcost.txt`. v99 row count not yet retrieved here; check during task 002. |
| Q7 | The 4-bit `runeword_id` padding in §5.3 is documented as "always written as 5". Does this hold for v105 fixtures with runewords? | Likely yes | §5.3 | Decode `UberSlapper.d2s` (a known runeword fixture) and inspect the 4-bit field after `runeword_id`. |
| Q8 | The 1-bit `realm_data_flag` in §5.6: does v105 ever set it to 1 in single-player saves? | Likely always 0 | §5.6 | Decode several non-simple items from the fixtures and check the bit. Preserve regardless. |
| Q9 | Does the `nr_of_items_in_sockets` field for a child item (recursively parsed in §10) ever exceed 0 in v105 fixtures? (i.e. nested sockets) | Likely no | §10 | Verify; the format permits it but in-game rules forbid socketing a socketed item. |
| Q10 | The `"jf"` section header in non-RotW v105 expansion saves: is it always present even with no merc hired? | Yes per `src/items/mod.rs` constants | §11.1 | Already confirmed by the empty-trailer constants. Documented here for clarity. |
| Q11 | Is the corpse-section 12-byte per-corpse block (`unk u32 + x u32 + y u32`) actually three `u32`s, or three `i32`s, or some other layout? | Unknown for v105 | §3.1, §11.2 | Find or generate a v105 fixture with at least one corpse item and decode. None of the listed v105 fixtures appear to have corpse items. |
| Q12 | Does the Delirium runeword's known v99 quirk (ID 2718 must remap to 48) still apply in v105? | Likely yes | §5.3 | Decode a v105 Delirium item if any fixture contains one. |

The task 003 plan should treat Q2, Q5, and Q6 as **must-verify-before-shipping**;
the rest can be left as known soft spots.

---

## Appendix A — Stat-Cost Bit Layout Table

The full per-stat bit-layout table for v105 lives in
`assets/excel/v105/itemstatcost.txt`. The header row (line 1) defines columns;
the columns required for parsing are:

| Index (1-based) | Header text | Used as |
|---|---|---|
| 1 | `Stat` | Stat name |
| 2 | `*ID` | Stat ID (the 9-bit value in the bitstream) |
| 15 | `Encode` | Encoding kind (0/blank, 2, 3, 4) — drives §7.5 / §7.6 |
| 21 | `Save Bits` | `sB` — value field width |
| 22 | `Save Add` | `sA` — bias |
| 23 | `Save Param Bits` | `sP` — optional param field width |

Sample rows (for orientation; full 369-row table to be embedded by task 002):

```
Stat                   *ID    Save Bits   Save Add   Save Param Bits   Encode
strength               0      8           32         (blank)           (blank)
energy                 1      7           32         (blank)           (blank)
maxhp                  7      9           32         (blank)           (blank)
item_armor_percent     16     9           0          (blank)           (blank)
armorclass             31     11          10         (blank)           (blank)
firemindam             48     8           0          (blank)           (blank)
durability             72     9           0          (blank)           (blank)
maxdurability          73     8           0          (blank)           (blank)
item_skillonhit        198    7           0          16                2
item_charged_skill     204    16          0          16                3
questitemdifficulty    356    2           0          (blank)           (blank)
```

**[Excel]** — values from `assets/excel/v105/itemstatcost.txt`. The complete
extracted table is the deliverable of task 002 (embedded data loading) and
will be consulted by task 003 (the parser). This appendix exists only as a
key to the columns; it is **not** intended to replace the file.

The `0x1FF` (decimal 511) sentinel value is **not** a row in the table —
it is reserved as the property-list terminator and must be excluded from any
generated stat-id-to-row mapping. **[v99-doc]**

---

## Appendix B — Per-Fixture Spot-Checks

This appendix is intentionally left empty for the initial revision. Hand-decoded
fixture spot-checks were not produced during task 001, in keeping with the
plan's allowance ("optional, kept short"). Task 003 (parser implementation)
will produce these as part of its golden-test development; they may be folded
back into this appendix in task 009 (documentation update) if useful.

The following fixtures from `assets/test/v105-real-characters/` are recommended
as the minimum spot-check set when implementing the parser, organised by item
shape:

| Shape | Fixture(s) |
|---|---|
| Simple items only | `NormalShortBows.d2s` |
| Magic / rare / crafted | `m_one.d2s`, `m_two.d2s`, `m_skillers.d2s` |
| Set / unique | `Liu.d2s`, `Shiva.d2s`, `Xen.d2s` |
| Runeword + sockets + inserted items | `UberSlapper.d2s` |
| Class-specific (Barb/Necro/Druid/Sorc/Amazon) | `BarbHelms.d2s`, `NecroHeads.d2s`, `DruidPelts.d2s`, `Circlets.d2s`, `Claws.d2s`, `NormalAmazonWep.d2s` |
| Throwables / quantity items | `Throws.d2s`, `NormalJavs.d2s` |
| Belt / boots / gloves | `Belts.d2s`, `Boots.d2s`, `Gloves.d2s` |
| Maximum-shape (full inventory + cube + stash + equipped + belt) | `Locker.d2s` |
| Corpse / golem item lists (empty-list framing) | any of the above |
| Shared stash | `SharedStashSoftCoreV2.d2i`, `ModernSharedStashSoftCoreV2.d2i` |

For each fixture, the §3 list framing should be decoded first (counts and
section magics), then one representative item per shape from each fixture
should be hand-decoded against the §4–§10 layout. Recording offset → field →
value tables for those items gives task 003 a regression baseline.

---

*End of document. Source HTML references retained at `docs/item-details.html`,
`docs/parsing-items.html`, `docs/moving-items.html`, and
`docs/v99-v105-details.html`. Excel reference data at `assets/excel/v105/`.
Source code conventions at `src/utils.rs`, `src/items/mod.rs`,
`src/character/v105.rs`, `src/format/decode.rs`.*
