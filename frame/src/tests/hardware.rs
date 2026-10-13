use super::*;

#[test]
fn stella_controls_and_aliases_remain_independent() {
    let mut keyboard = KeyboardConsole::default();
    for key in [
        Keycode::F1,
        Keycode::Tab,
        Keycode::Return,
        Keycode::Kp8,
        Keycode::Space,
        Keycode::F5,
        Keycode::F7,
        Keycode::F4,
    ] {
        keyboard.key_down(key);
    }
    let controls = keyboard.sample();
    assert!(controls.select && controls.reset && controls.left.direction.up && controls.left.fire);
    assert!(!controls.color);
    assert_eq!(controls.left_difficulty, Difficulty::A);
    assert_eq!(controls.right_difficulty, Difficulty::A);
    keyboard.pressed.remove(&Keycode::Tab);
    assert!(keyboard.sample().select);
    keyboard.pressed.remove(&Keycode::F1);
    assert!(!keyboard.sample().select);
    keyboard.key_down(Keycode::F2);
    keyboard.pressed.remove(&Keycode::Return);
    assert!(keyboard.sample().reset);
    keyboard.key_down(Keycode::F3);
    keyboard.key_down(Keycode::F6);
    keyboard.key_down(Keycode::F8);
    assert_eq!(keyboard.sample().left_difficulty, Difficulty::B);
    assert_eq!(keyboard.sample().right_difficulty, Difficulty::B);
    assert!(keyboard.sample().color);
}
