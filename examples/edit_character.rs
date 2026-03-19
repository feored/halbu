use halbu::quests::QuestFlag;
use halbu::{CompatibilityChecks, Save, Strictness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "assets/test/Warlock_v105.d2s";
    let output_name = "WarlockDemo";
    let output_dir = "target/example-output";
    let output_path = format!("{output_dir}/{output_name}.d2s");

    let bytes = std::fs::read(input_path)?;
    let parsed = Save::parse(&bytes, Strictness::Strict)?;
    if !parsed.issues.is_empty() {
        return Err(format!("fixture parsed with issues: {:?}", parsed.issues).into());
    }

    let mut save = parsed.save;
    let target_format = save.format();

    println!(
        "Loaded {:?} / {} / lvl {}",
        target_format,
        save.character.name,
        save.character.level()
    );

    // Keep the file name and in-game name aligned.
    save.character.name = output_name.to_string();

    // Keep character.level and attributes.level in sync.
    save.set_level(75);

    // Main stats.
    save.attributes.strength.value = 220;
    save.attributes.dexterity.value = 175;
    save.attributes.vitality.value = 260;
    save.attributes.energy.value = 110;
    save.attributes.statpts.value = 25;
    save.attributes.newskills.value = 10;
    save.attributes.gold.value = 200_000;
    save.attributes.goldbank.value = 2_500_000;

    // These stats are fixed-point in the save file (Q8), so the helpers take in-game values.
    save.attributes.set_max_hp(2200);
    save.attributes.set_hp(2200);
    save.attributes.set_max_mana(900);
    save.attributes.set_mana(900);
    save.attributes.set_max_stamina(1200);
    save.attributes.set_stamina(1200);

    // Use the D2R name lookup when it exists.
    // Fall back to raw slot indexes when it does not.
    if save.skills.set_by_name_d2r(save.character.class, "Bash", 20).is_err() {
        save.skills.set(0, 20);
    }
    if save.skills.set_by_name_d2r(save.character.class, "Battle Orders", 20).is_err() {
        save.skills.set(23, 20);
    }
    if save.skills.set_by_name_d2r(save.character.class, "Whirlwind", 20).is_err() {
        save.skills.set(25, 20);
    }

    // Give Act I all waypoints.
    save.waypoints.normal.act1.set_all(true);

    // Edit one quest state.
    save.quests.normal.act1.q1.state.insert(QuestFlag::Started);

    let act1_catacombs = save.waypoints.normal.act1.get_by_index(8)?;
    let quest_started = save.quests.normal.act1.q1.state.contains(&QuestFlag::Started);

    // Write back to the same detected format.
    let output_bytes = save.encode_for(target_format, CompatibilityChecks::Enforce)?;
    std::fs::create_dir_all(output_dir)?;
    std::fs::write(&output_path, output_bytes)?;

    println!("Wrote {output_path}");
    println!(
        "Now: {} / lvl {}",
        save.character.name,
        save.character.level()
    );
    println!(
        "Stats: str={} dex={} vit={} ene={}",
        save.attributes.strength.value,
        save.attributes.dexterity.value,
        save.attributes.vitality.value,
        save.attributes.energy.value
    );
    println!(
        "Resources: hp={} mana={} stamina={}",
        save.attributes.get_max_hp(),
        save.attributes.get_max_mana(),
        save.attributes.get_max_stamina()
    );
    println!("Act I / Catacombs waypoint: {act1_catacombs}");
    println!("Quest q1 started: {quest_started}");

    Ok(())
}
