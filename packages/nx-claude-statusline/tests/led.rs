use nx_claude_statusline::led::{agent_state, band, program};

#[test]
fn bands_switch_at_50_70_and_85() {
    assert_eq!(band(49.9), "#00FF66");
    assert_eq!(band(50.0), "#FFD000");
    assert_eq!(band(70.0), "#FF6A00");
    assert_eq!(band(85.0), "#FF0000");
}

#[test]
fn working_program_cycles_meters_on_led1_and_blinks_led0() {
    let text = program("Working", &["#00FF66", "#FF0000"], 16);
    assert_eq!(
        text,
        "brightness 16\n\
         0:#00E5FF 1:#00FF66 400ms ease\n0:#000000 1:#00FF66 400ms ease\n\
         0:#00E5FF 1:#00FF66 400ms ease\n0:#000000 1:#00FF66 400ms ease\n\
         0:#00E5FF 1:#000000 200ms ease\n\
         0:#000000 1:#FF0000 400ms ease\n0:#00E5FF 1:#FF0000 400ms ease\n\
         0:#000000 1:#FF0000 400ms ease\n0:#00E5FF 1:#FF0000 400ms ease\n\
         0:#000000 1:#000000 200ms ease\n\
         0:#00E5FF 1:#000000 400ms ease\n0:#000000 1:#000000 400ms ease\n\
         repeat\n"
    );
    assert!(text.len() <= 512 && text.lines().count() <= 20);
    assert!(!program("Done", &["#00FF66"], 255).contains("#000000 1:#00FF66"));
}

#[test]
fn agent_state_takes_the_last_state_line_and_ignores_lid_state() {
    let tail = "t state=Ask\nt lid_state=closed\nt leds=x\nt state=Working\nt lid_state=open\n";
    assert_eq!(agent_state(tail), Some("Working"));
    assert_eq!(agent_state("t leds=x\n"), None);
}
