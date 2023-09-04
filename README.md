# Halbu

A .d2s file parsing library written in Rust.

⚠ NPCs and Items section are not yet supported.

[Notes](./notes.md) regarding D2R with some useful information regarding quests in particular.

### Usage

```rs
use halbu::{quests::QuestFlag, waypoints::Waypoint, Class, Save};

fn main() {
    let save_file =
        std::fs::read("C:\\Users\\Example\\Saved Games\\Diablo II Resurrected\\Jamella.d2s")
            .unwrap();

    let mut save = Save::parse(&save_file);

    // Change class, name, etc
    save.character.class = Class::Paladin;
    save.character.name = String::from("Halbu");
    save.character.level = 47;
    save.attributes.level.value = 47;
    // Warning: save.character.level and save.attributes.level must
    // be the same or the game won't load!
    println!(
        "{0}, Level {1} {2}",
        save.character.name, save.character.level, save.character.class
    );

    // Change mercenary stats
    // Refer to notes.md for a table with name/variant ID
    save.character.mercenary.name_id = 3;
    save.character.mercenary.variant_id = 34;

    // Set an attribute
    save.attributes.strength.value = 156;

    // Attribute names are taken from itemstatcosts.txt
    save.attributes.newskills.value = 5;

    // Some attributes are stored as fixed point numbers in 21 bits,
    // where the first 13 bits are the integer part and the last 8 the decimal
    // For those attributes (Current/Max HP, Mana & Stamina), you must multiply
    // the value by 256 to get the value you want
    save.attributes.maxmana.value = 200 * 256;
    println!(
        "Max mana: {}",
        save.attributes.maxmana.value as f64 / 256f64
    );

    // Acquire all waypoints in an act
    save.waypoints.normal.act1.set_all(true);
    println!("Normal:\n{}", save.waypoints.normal.act1);

    // Set all waypoints in a difficulty
    // Warning: Not a good idea to set Rogue Encampment to false
    save.waypoints.hell.set_all(false);

    // Get/set whether a specific waypoint is acquired
    // Waypoints are a numbered 0-8 (0-2 for Act IV)
    save.waypoints.hell.act4.set_num(1, true);
    println!("Hell Act IV WP 1: {}", save.waypoints.hell.act4.get_num(1));

    save.waypoints
        .hell
        .act4
        .set(Waypoint::CityOfTheDamned, false);
    println!(
        "Hell Act IV WP 1: {}",
        save.waypoints.hell.act4.get(Waypoint::CityOfTheDamned)
    );

    // Set all skills to 20
    save.skills.set_all(20);
    println!("{}", save.skills);

    // Set the skill points of a given skill to 0
    save.skills.set(17, 0);
    println!("Skillpoints: {}", save.skills.get(17));

    // A quest is a struct with a single member State which is a hashset

    // Clear all flags
    // Warning: The quest numbers may not be what you think! Refer to notes.md.
    save.quests.hell.act1.q1.state.clear();
    println!("Hell Act I Q1 State: {}", save.quests.hell.act1.q1);

    // The flag names are from D2MOO. Refer to notes.md for a flagname <> bit table.
    save.quests
        .hell
        .act1
        .q1
        .state
        .insert(QuestFlag::RewardGranted);
    println!(
        "Hell Act I Q1 Completed: {}",
        save.quests
            .hell
            .act1
            .q1
            .state
            .contains(&QuestFlag::RewardGranted),
    );
    // Save the file
    // Warning: The file name must match the character's name!
    std::fs::write(
        "C:\\Users\\Example\\Saved Games\\Diablo II Resurrected\\Halbu.d2s",
        save.to_bytes(),
    )
    .unwrap();
}

```




#### Resources

These resources have helped me understand the .d2s format. Many thanks to their authors for the work they've done!

* http://user.xmission.com/~trevin/DiabloIIv1.09_File_Format.shtm
* https://github.com/dschu012/D2SLib (Unless you specifically need a rust library, you should probably use this.)
* https://raw.githubusercontent.com/oaken-source/pyd2s/master/docs/d2s_save_file_format_1.13d.txt
* https://github.com/WalterCouto/D2CE/blob/main/d2s_File_Format.md
* https://github.com/krisives/d2s-format
* https://github.com/nokka/d2s/
* https://github.com/ThePhrozenKeep/D2MOO
* https://d2mods.info/forum/kb/index?c=4
