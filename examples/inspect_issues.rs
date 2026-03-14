use halbu::{Save, Strictness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start from a known-good fixture.
    let bytes = std::fs::read("assets/test/Joe.d2s")?;

    // Strict mode is what you want for CI or hard validation.
    let strict = Save::parse(&bytes, Strictness::Strict)?;
    println!(
        "Strict parse OK: format={:?}, name={}",
        strict.save.format(),
        strict.save.character.name
    );

    // Build a broken payload on purpose so we can inspect non-fatal issues.
    // - byte 0 breaks signature
    // - truncation simulates a damaged/incomplete file
    let mut damaged = bytes.clone();
    damaged[0] = 0x00;
    damaged.truncate(220);

    // Lax mode keeps going and gives you a list of issues.
    let lax = Save::parse(&damaged, Strictness::Lax)?;
    println!("\nLax parse issues: {}", lax.issues.len());
    for (index, issue) in lax.issues.iter().enumerate() {
        println!(
            "#{index} [{:?}/{:?}] section={:?} offset={:?} expected={:?} found={:?}\n  {}",
            issue.severity,
            issue.kind,
            issue.section,
            issue.offset,
            issue.expected,
            issue.found,
            issue.message
        );
    }

    // Same payload in strict mode should fail early.
    match Save::parse(&damaged, Strictness::Strict) {
        Ok(_) => println!("\nUnexpected: strict parse accepted damaged payload."),
        Err(error) => println!("\nStrict parse error: {error}"),
    }

    Ok(())
}
