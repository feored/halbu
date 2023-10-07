#[cfg(test)]
mod tests {
    use crate::items::*;

    #[test]
    fn items_header_test() {
        let bytes: [u8; 10] = [0x10, 0x0, 0x80, 0x0, 0x4D, 0x04, 0x40, 0xBC, 0x19, 0xF2];
        let mut byte_data = ByteIO::new(&bytes);
        // Header { identified: true, broken: false, socketed: false, ear: false, starter_gear: false, compact: false, ethereal: false,
        // personalized: false, runeword: false, status: Equipped, slot: Helmet, column: 1, row: 0, storage: None, base: "cap ", socketed_count: 0 }
        let parsed_result = Header::parse(&mut byte_data).unwrap();
        let expected_result = Header {
            identified: true,
            status: Status::Equipped,
            slot: Slot::Helmet,
            column: 1,
            base: String::from("cap "),
            ..Default::default()
        };
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn item_mods_test() {
        let bytes: [u8; 14] =
            [0x10, 0x34, 0x9C, 0x70, 0x96, 0xA5, 0x92, 0xD, 0x26, 0x28, 0xF8, 0xD, 0xFF, 0x1];
        let mut byte_data = ByteIO::new(&bytes);
        // mods: mods: [[ItemMod { base: Mod { key: 16, value: 26, name: "item_armor_percent" }, linked_mods: [], param: None },
        // ItemMod { base: Mod { key: 39, value: 6, name: "fireresist" }, linked_mods: [], param: None }, ItemMod { base: Mod { key: 89, value: 1,
        // name:"item_lightradius" }, linked_mods: [], param: None }, ItemMod { base: Mod { key: 201, value: 3, name: "item_skillongethit" },
        // linked_mods: [], param: Some(5139) }, ItemMod { base: Mod { key: 252, value: 3, name: "item_replenish_durability" }, linked_mods: [], param: None }]], set_item_mask: 0 }),

        let parsed_result = ItemMod::parse(&mut byte_data).unwrap();
        let generated_result = ItemMod::to_bytes(&parsed_result);
        assert_eq!(bytes.to_vec(), generated_result.data);
    }

    #[test]
    fn item_complete_test() {
        let bytes: [u8; 49] = [
            16, 0, 128, 0, 0, /*5 */
            140, 149, 227, 21, 146, 153, 240, 87, 151, 69, 18, 200, 1, 198, 131, 15, 128, 70, 0,
            237, 195, 18, 146, 17, 38, 48, 254, 119, 105, 253, 255, 32, 136, 255, 60, 20, 255, 125,
            40, 254, 27, 18, 255, 1,
        ];
        let mut byte_data = ByteIO::new(&bytes);
        let example_item = Item {
            header: Header {
                identified: true,
                broken: false,
                socketed: false,
                ear: false,
                picked_up_since_last_save: false,
                starter_gear: false,
                compact: false,
                ethereal: false,
                personalized: false,
                runeword: false,
                status: Status::Stored,
                slot: Slot::None,
                column: 3,
                row: 6,
                storage: Storage::Stash,
                base: String::from("xhg "),
                socketed_count: 0,
            },
            data: Some(ExtendedItem {
                id: 2885176521,
                level: 75,
                quality: Quality::Set(73),
                custom_graphics_id: None,
                auto_mod: None,
                name_prefix: None,
                name_suffix: None,
                prefixes: [0, 0, 0],
                suffixes: [0, 0, 0],
                personalized_name: None,
                runeword_id: None,
                realm_data: None,
                defense: Some(47),
                durability_max: 24,
                durability_current: Some(15),
                quantity: None,
                total_sockets: None,
                mods: vec![
                    vec![
                        ItemMod {
                            base: Mod { key: 0, value: 20, name: String::from("strength") },
                            linked_mods: Vec::<Mod>::new(),
                            param: None,
                        },
                        ItemMod {
                            base: Mod { key: 2, value: 20, name: String::from("dexterity") },
                            linked_mods: Vec::<Mod>::new(),
                            param: None,
                        },
                        ItemMod {
                            base: Mod { key: 31, value: 65, name: String::from("armorclass") },
                            linked_mods: Vec::<Mod>::new(),
                            param: None,
                        },
                        ItemMod {
                            base: Mod {
                                key: 201,
                                value: 12,
                                name: String::from("item_skillongethit"),
                            },
                            linked_mods: Vec::<Mod>::new(),
                            param: Some(2436),
                        },
                    ],
                    vec![ItemMod {
                        base: Mod {
                            key: 93,
                            value: 25,
                            name: String::from("item_fasterattackrate"),
                        },
                        linked_mods: Vec::<Mod>::new(),
                        param: None,
                    }],
                    vec![ItemMod {
                        base: Mod { key: 31, value: 120, name: String::from("armorclass") },
                        linked_mods: Vec::<Mod>::new(),
                        param: None,
                    }],
                    vec![ItemMod {
                        base: Mod { key: 60, value: 10, name: String::from("lifedrainmindam") },
                        linked_mods: Vec::<Mod>::new(),
                        param: None,
                    }],
                    vec![ItemMod {
                        base: Mod { key: 62, value: 10, name: String::from("manadrainmindam") },
                        linked_mods: Vec::<Mod>::new(),
                        param: None,
                    }],
                    vec![ItemMod {
                        base: Mod { key: 134, value: 2, name: String::from("item_freeze") },
                        linked_mods: Vec::<Mod>::new(),
                        param: None,
                    }],
                ],
                set_item_mask: 31,
            }),
            socketed_items: Vec::<Item>::new(),
        };

        let parsed_result = Item::parse(&mut byte_data).unwrap();
        assert_eq!(example_item, parsed_result);
        // println!("item_complete_test:{0:?}", parsed_result);
        let generated_result = Item::to_bytes(&parsed_result);
        let other_generated_result = example_item.to_bytes();
        println!("item_complete_test:{0:?}", other_generated_result);
        //let new_parsed = Item::parse(&mut ByteIO::new(&generated_result)).unwrap();
        //assert_eq!(parsed_result, new_parsed);
        assert_eq!(bytes.to_vec(), generated_result);
    }
}
