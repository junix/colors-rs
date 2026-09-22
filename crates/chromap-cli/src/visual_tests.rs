use super::{should_enable_ansi, ColorPolicy};

#[test]
fn automatic_color_requires_a_capable_tty() {
    assert!(should_enable_ansi(
        ColorPolicy::Auto,
        false,
        false,
        true,
        false,
        false,
    ));
    assert!(!should_enable_ansi(
        ColorPolicy::Auto,
        false,
        false,
        false,
        false,
        false,
    ));
    assert!(!should_enable_ansi(
        ColorPolicy::Auto,
        false,
        false,
        true,
        true,
        false,
    ));
    assert!(!should_enable_ansi(
        ColorPolicy::Auto,
        false,
        false,
        true,
        false,
        true,
    ));
}

#[test]
fn plain_and_json_override_forced_color() {
    assert!(!should_enable_ansi(
        ColorPolicy::Always,
        true,
        false,
        true,
        false,
        false,
    ));
    assert!(!should_enable_ansi(
        ColorPolicy::Always,
        false,
        true,
        true,
        false,
        false,
    ));
}
