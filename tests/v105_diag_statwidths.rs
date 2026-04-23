//! Print stat row widths for high-id stats observed in failing fixtures.

#[test]
fn dump_high_stat_widths() {
    let ids = [127u16, 138, 252, 256, 275, 320, 322, 354, 360, 448, 454];
    for id in ids {
        match halbu::__diag_statcost_row(id) {
            Some((name, sb, sa, sp, enc)) => {
                eprintln!("stat {id} name={name} save_bits={sb} save_add={sa} save_param_bits={sp} encode={enc}");
            }
            None => eprintln!("stat {id} NOT IN TABLE"),
        }
    }
}
