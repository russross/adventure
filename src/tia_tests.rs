use super::*;

#[test]
fn hidden_pixels_still_latch_contacts_and_delayed_rows_commit_on_first_object() {
    let mut tia = Tia::default();
    tia.blank(false);
    tia.background(8);
    tia.foreground(28);
    tia.foreground_priority(true);
    tia.background_pattern(PlayfieldRow::new(0xf0, 0, 0));
    tia.first_object.set_color(72);
    tia.second_object.set_color(104);
    tia.first_object.reposition(0);
    tia.second_object.reposition(0);
    tia.player.reposition(0);
    tia.player.enable(true);
    tia.second_object.delay_rows(true);
    tia.second_object.set_row(0x80);
    tia.horizontal_sync();
    assert!(!tia.contacts.objects_touch);
    assert!(tia.contacts.player_walls);

    tia.first_object_row(0x80);
    tia.horizontal_sync();
    assert!(tia.contacts.objects_touch);
    assert_eq!(tia.contacts.player_objects, [true, true]);
    assert_eq!(tia.frame.pixels[WIDTH], 28);
    tia.clear_contacts();
    assert_eq!(tia.contacts, Contacts::default());
}

#[test]
fn patterns_mirror_or_repeat_and_duplicate_scaled_sprites() {
    let mut tia = Tia::default();
    tia.blank(false);
    tia.foreground(28);
    tia.background_pattern(PlayfieldRow::new(0x10, 0, 0));
    tia.horizontal_sync();
    assert_eq!(&tia.frame.pixels[156..160], &[28; 4]);
    assert_eq!(tia.frame.pixels[80], 0);
    tia.mirror_walls(false);
    tia.horizontal_sync();
    assert_eq!(&tia.frame.pixels[WIDTH + 80..WIDTH + 84], &[28; 4]);

    tia.background_pattern(PlayfieldRow::default());
    tia.first_object.reposition(10);
    tia.first_object.set_color(72);
    tia.first_object.scale(Scale::Double);
    tia.first_object.duplicate(Copies::TripleMedium);
    tia.first_object_row(0x80);
    tia.horizontal_sync();
    for x in [11, 43, 75] {
        assert_eq!(
            &tia.frame.pixels[WIDTH * 2 + x..WIDTH * 2 + x + 2],
            &[72; 2]
        );
    }
    assert_eq!(tia.frame.pixels[WIDTH * 2 + 10], 0);
    assert_eq!(tia.frame.pixels[WIDTH * 2 + 13], 0);
}

#[test]
fn enlarged_sprite_edges_shift_and_wrap_in_both_slots() {
    let cases: &[(u8, Scale, &[usize])] = &[
        (158, Scale::Normal, &[5, 158]),
        (158, Scale::Double, &[0, 13, 14, 159]),
        (158, Scale::Quadruple, &[0, 1, 2, 27, 28, 29, 30, 159]),
        (159, Scale::Double, &[0, 1, 14, 15]),
        (159, Scale::Quadruple, &[0, 1, 2, 3, 28, 29, 30, 31]),
        (255, Scale::Double, &[96, 97, 110, 111]),
        (255, Scale::Quadruple, &[96, 97, 98, 99, 124, 125, 126, 127]),
        (159, Scale::Normal, &[6, 159]),
    ];

    // Both ends of a sparse bitmap expose clipping, bit order, and the screen seam.
    for second_slot in [false, true] {
        let mut tia = Tia::default();
        tia.blank(false);
        for &(position, scale, expected) in cases {
            let sprite = if second_slot {
                &mut tia.second_object
            } else {
                &mut tia.first_object
            };
            sprite.reposition(position);
            sprite.scale(scale);
            sprite.set_color(72);
            sprite.set_row(0x81);

            // Reuse the drawable so returning to normal size also removes the displacement.
            tia.begin_picture();
            tia.horizontal_sync();
            let painted: Vec<usize> = tia.frame.pixels[..WIDTH]
                .iter()
                .enumerate()
                .filter_map(|(x, &color)| (color == 72).then_some(x))
                .collect();
            assert_eq!(
                painted, expected,
                "{position}, {scale:?}, second={second_slot}"
            );
        }
    }
}

#[test]
fn wrapped_wide_sprite_contacts_follow_shifted_pixels_even_behind_walls() {
    for (x, touching) in [(159, false), (0, true), (3, true), (4, false)] {
        let mut tia = Tia::default();
        tia.blank(false);
        tia.foreground(28);
        tia.foreground_priority(true);
        tia.background_pattern(PlayfieldRow::new(0xf0, 0, 0));
        tia.player.set_width(1);
        tia.player.enable(true);
        tia.player.reposition(x);

        // The enlarged delayed sprite occupies pixels 0..3 after wrapping from x=159.
        tia.second_object.reposition(159);
        tia.second_object.scale(Scale::Quadruple);
        tia.second_object.delay_rows(true);
        tia.second_object.set_row(0x80);
        tia.first_object.reposition(x);
        tia.first_object_row(0x80);
        tia.horizontal_sync();
        assert_eq!(tia.contacts.player_objects, [true, touching], "x={x}");
        assert_eq!(tia.contacts.objects_touch, touching, "x={x}");
        assert_eq!(&tia.frame.pixels[..4], &[28; 4]);
    }
}
