use halbu::{Save, Strictness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start from a known good file.
    let bytes = std::fs::read("assets/test/Joe.d2s")?;

    // Strict mode fails fast on invalid data.
    let strict = Save::parse(&bytes, Strictness::Strict)?;
    println!(
        "Strict parse OK: format={:?}, name={}",
        strict.save.format(),
        strict.save.character.name
    );

    // Validation is separate from parsing, so check the save on its own.
    let validation = strict.save.validate();
    println!("\nValidation issues: {}", validation.issues.len());
    for (index, issue) in validation.issues.iter().enumerate() {
        println!("#{index} [{:?}] {}", issue.code, issue.message);
    }

    // Break the payload on purpose so we can inspect non-fatal issues.
    // - byte 0 breaks signature
    // - truncation simulates a damaged or incomplete file
    let mut damaged = bytes.clone();
    damaged[0] = 0x00;
    damaged.truncate(220);

    // Lax mode keeps going and collects issues.
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

    // Strict mode should reject the damaged payload.
    match Save::parse(&damaged, Strictness::Strict) {
        Ok(_) => println!("\nUnexpected: strict parse accepted damaged payload."),
        Err(error) => println!("\nStrict parse error: {error}"),
    }

    Ok(())
}
