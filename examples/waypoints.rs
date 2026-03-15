use halbu::waypoints::{Waypoint, WaypointError};
use halbu::{Save, Strictness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("assets/test/Joe.d2s")?;
    let parsed = Save::parse(&bytes, Strictness::Strict)?;
    let mut save = parsed.save;

    // Read by index (Hell Act I, Catacombs).
    let catacombs_was_unlocked = save.waypoints.hell.act1.get_by_index(8)?;
    println!("Hell Act I / Catacombs unlocked: {catacombs_was_unlocked}");

    // Set by waypoint id.
    save.waypoints.hell.act1.set(Waypoint::Catacombs, true)?;
    let catacombs_is_unlocked = save.waypoints.hell.act1.get(Waypoint::Catacombs)?;
    println!("Hell Act I / Catacombs now unlocked: {catacombs_is_unlocked}");

    // Set by index (Normal Act IV has indices 0..=2).
    save.waypoints.normal.act4.set_by_index(2, true)?;
    println!(
        "Normal Act IV / River of Flames unlocked: {}",
        save.waypoints.normal.act4.get_by_index(2)?
    );

    // Bulk update one whole difficulty.
    save.waypoints.nightmare.set_all(true);
    println!(
        "Nightmare Act II / Sewers unlocked after set_all: {}",
        save.waypoints.nightmare.act2.get_by_index(1)?
    );

    // Wrong-act usage is explicit.
    match save.waypoints.normal.act1.set(Waypoint::LutGholein, true) {
        Err(WaypointError::WrongAct { waypoint, expected, actual }) => {
            println!("WrongAct: {waypoint:?} belongs to {actual:?}, expected {expected:?}.")
        }
        Ok(_) => println!("Unexpected: wrong-act set was accepted."),
        Err(error) => println!("Unexpected waypoint error: {error}"),
    }

    // Out-of-range index usage is explicit.
    match save.waypoints.normal.act4.get_by_index(3) {
        Err(WaypointError::IndexOutOfRange { act, index, max_index }) => {
            println!("IndexOutOfRange: {act:?} index {index} (max {max_index}).")
        }
        Ok(_) => println!("Unexpected: out-of-range read was accepted."),
        Err(error) => println!("Unexpected waypoint error: {error}"),
    }

    Ok(())
}
