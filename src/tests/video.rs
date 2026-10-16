use super::*;

#[test]
fn partial_row_blanking_removes_pixels_and_contacts_at_the_exact_edge() {
    let mut picture = Picture::default();
    picture.appearance.background = 8;
    picture.appearance.player_width = 1;
    picture.player_visible[HEIGHT - 1] = true;
    picture.objects[0].rows[HEIGHT - 1] = 0xff;
    picture.visible_widths[HEIGHT - 1] = 5;
    picture.player_x = 4;
    let mut video = Video::default();
    video.paint(&picture);
    assert!(video.contacts.player_objects[0]);
    picture.player_x = 5;
    video.paint(&picture);
    assert!(!video.contacts.player_objects[0]);
    assert!(
        video.frame.pixels[(HEIGHT - 1) * WIDTH + 5..]
            .iter()
            .all(|pixel| *pixel == 0)
    );
    assert_eq!(video.frame.pixels, reference_picture(&picture).0);
}

#[test]
fn moving_and_erasing_sprites_recomposes_only_affected_rows() {
    let mut picture = Picture::default();
    picture.objects[0].line.color = 72;
    picture.objects[0].rows[10] = 0xff;
    let mut video = Video::default();
    video.paint(&picture);
    assert_eq!(
        video.frame.changed_rows.as_slice(),
        std::slice::from_ref(&(0..HEIGHT))
    );
    video.paint(&picture);
    assert!(video.frame.changed_rows.is_empty());

    // Erase the old location and paint the new one without touching intervening rows.
    picture.objects[0].rows[10] = 0;
    picture.objects[0].rows[12] = 0xff;
    video.paint(&picture);
    assert_eq!(video.frame.changed_rows, [10..11, 12..13]);
    assert_eq!(video.frame.pixels, reference_picture(&picture).0);

    // Global color changes and erasure still reconstruct the full reference picture.
    picture.appearance.background = 8;
    video.paint(&picture);
    assert_eq!(
        video.frame.changed_rows.as_slice(),
        std::slice::from_ref(&(0..HEIGHT))
    );
    assert_eq!(video.frame.pixels, reference_picture(&picture).0);
    picture.objects[0].rows[12] = 0;
    video.paint(&picture);
    assert_eq!(
        video.frame.changed_rows.as_slice(),
        std::slice::from_ref(&(12..13))
    );
    assert_eq!(video.frame.pixels, reference_picture(&picture).0);
}

#[test]
fn collision_geometry_can_run_without_painting_and_survive_replacement_artwork() {
    let picture = Picture {
        player_visible: [true; HEIGHT],
        appearance: Appearance {
            player_width: 4,
            ..Appearance::default()
        },
        objects: [Object {
            line: ObjectLine::default(),
            rows: [0xff; HEIGHT],
        }; 2],
        ..Picture::default()
    };
    let geometry = Geometry::prepare(&picture);
    let contacts = geometry.contacts();
    assert_eq!(contacts.player_objects, [true; 2]);
    assert!(contacts.objects_touch);

    // An empty visual replacement has no bearing on the original contact geometry.
    let replacement = Picture::default();
    let mut video = Video {
        contacts,
        ..Video::default()
    };
    video.compose(&replacement, &Geometry::prepare(&replacement));
    assert_eq!(video.contacts, contacts);
    assert!(video.frame.pixels.iter().all(|pixel| *pixel == 0));
}

#[test]
fn complete_objects_preserve_hidden_contacts_and_wide_seam_geometry() {
    let mut video = Video::default();
    let mut picture = Picture {
        appearance: Appearance {
            foreground: 28,
            walls_in_front: true,
            player_width: 4,
            mirrored: true,
            barriers: [true; 2],
            ..Appearance::default()
        },
        walls: [PlayfieldRow::new(0x10, 0, 0); HEIGHT],
        objects: [Object {
            line: ObjectLine {
                x: 159,
                width: SpriteWidth::Wide,
                color: 72,
                bits: 0,
            },
            rows: [0x81; HEIGHT],
        }; 2],
        player_x: 159,
        player_visible: [true; HEIGHT],
        visible_widths: [WIDTH as u8; HEIGHT],
    };
    video.paint(&picture);
    assert_eq!(video.contacts.player_objects, [true, true]);
    assert!(video.contacts.objects_touch);
    assert!(video.contacts.player_walls);
    assert_eq!(video.frame.pixels[159], 28);
    assert_eq!(video.frame.pixels[0], 28);

    // Remove foreground coverage and distinguish the overlapping object colors.
    picture.player_visible.fill(false);
    picture.walls.fill(PlayfieldRow::default());
    picture.objects[1].line.color = 94;
    video.paint(&picture);
    for y in [0, HEIGHT - 1] {
        for x in [0, 1, 2, 27, 28, 29, 30, 159, 11] {
            assert_eq!(video.frame.pixels[y * WIDTH + x], 72);
        }
        assert_eq!(video.frame.pixels[y * WIDTH + 158], 0);
        assert_eq!(video.frame.pixels[y * WIDTH + 148], 94);
    }
    assert_eq!(video.contacts.player_objects, [false; 2]);
    assert!(video.contacts.objects_touch);
    video.paint(&Picture::default());
    assert_eq!(video.contacts, Contacts::default());
}

#[test]
fn wall_cells_mirror_or_repeat_across_the_complete_picture() {
    let mut video = Video::default();
    let mut picture = Picture {
        walls: [PlayfieldRow::new(0x10, 0, 0); HEIGHT],
        appearance: Appearance {
            foreground: 28,
            mirrored: true,
            ..Appearance::default()
        },
        ..Picture::default()
    };
    video.paint(&picture);
    assert_eq!(&video.frame.pixels[156..160], &[28; 4]);
    assert_eq!(video.frame.pixels[80], 0);
    picture.appearance.mirrored = false;
    video.paint(&picture);
    assert_eq!(&video.frame.pixels[WIDTH + 80..WIDTH + 84], &[28; 4]);
}

// An independent pixel compositor checks overlap geometry before selecting visible colors.
fn reference_picture(picture: &Picture) -> (Vec<u8>, Contacts) {
    let mut pixels = vec![0; WIDTH * HEIGHT];
    let mut contacts = Contacts::default();
    let appearance = picture.appearance;
    for (index, pixel) in pixels.iter_mut().enumerate() {
        let y = index / WIDTH;
        let x = index % WIDTH;
        if x >= usize::from(picture.visible_widths[y]) {
            continue;
        }
        let relative =
            |origin: u8| (x as i16 - i16::from(origin)).rem_euclid(WIDTH as i16) as usize;
        let walls = picture.walls[y].covers(x, appearance.mirrored);
        let player = picture.player_visible[y]
            && relative(picture.player_x) < usize::from(appearance.player_width);
        let objects = [0, 1].map(|slot| {
            let object = &picture.objects[slot];
            let scale = match object.line.width {
                SpriteWidth::Normal => 1,
                SpriteWidth::Wide => 4,
            };
            let offset = relative(object.line.x);
            offset < 8 * scale && object.rows[y] & (0x80 >> (offset / scale)) != 0
        });

        // Barriers share colors with their slots but never count as bitmap contacts.
        let barriers =
            [0, 1].map(|slot| appearance.barriers[slot] && relative([11, 148][slot]) < 2);
        contacts.player_walls |= player && walls;
        for slot in 0..2 {
            contacts.player_objects[slot] |= player && objects[slot];
            contacts.player_barriers[slot] |= player && barriers[slot];
        }
        contacts.objects_touch |= objects[0] && objects[1];
        let foreground = walls || player;
        *pixel = if appearance.walls_in_front && foreground {
            appearance.foreground
        } else if objects[0] || barriers[0] {
            picture.objects[0].line.color
        } else if objects[1] || barriers[1] {
            picture.objects[1].line.color
        } else if foreground {
            appearance.foreground
        } else {
            appearance.background
        };
    }
    (pixels, contacts)
}

#[test]
fn complete_pictures_match_pixel_reference_at_seams_and_hidden_overlaps() {
    let mut video = Video::default();
    for (case, x) in [0u8, 1, 11, 148, 159, 160, 175, 255]
        .into_iter()
        .enumerate()
    {
        for first_width in [SpriteWidth::Normal, SpriteWidth::Wide] {
            for second_width in [SpriteWidth::Normal, SpriteWidth::Wide] {
                for mirrored in [false, true] {
                    for walls_in_front in [false, true] {
                        for batch in 0..2 {
                            let mut picture = Picture {
                                appearance: Appearance {
                                    background: 8,
                                    foreground: 28,
                                    mirrored,
                                    walls_in_front,
                                    player_width: [0, 1, 4, 8, 160, 255][case % 6],
                                    barriers: [case & 1 != 0, case & 2 != 0],
                                },
                                player_x: x.wrapping_sub(2),
                                ..Picture::default()
                            };

                            // Two pictures cover every bitmap byte, including empty and solid rows.
                            picture.objects[0].line = ObjectLine {
                                x,
                                color: 72,
                                width: first_width,
                                bits: 0,
                            };
                            picture.objects[1].line = ObjectLine {
                                x: if mirrored { x } else { x.wrapping_add(3) },
                                color: 94,
                                width: second_width,
                                bits: 0,
                            };
                            for y in 0..HEIGHT {
                                let bits = (y + batch * HEIGHT) as u8;
                                picture.objects[0].rows[y] = bits;
                                picture.objects[1].rows[y] = bits.reverse_bits();
                                picture.player_visible[y] = y % 3 != 0;
                                picture.walls[y] = if y / 16 % 2 == 0 {
                                    PlayfieldRow::new(0x90, 0xa5, 0x81)
                                } else {
                                    PlayfieldRow::default()
                                };
                            }
                            let (pixels, contacts) = reference_picture(&picture);
                            video.paint(&picture);
                            assert_eq!(video.frame.pixels, pixels, "case {case}, batch {batch}");
                            assert_eq!(video.contacts, contacts, "case {case}, batch {batch}");
                            assert_eq!(video.frame.painted_lines, HEIGHT);
                        }
                    }
                }
            }
        }
    }
}

// Invisible geometry changes still invalidate a retained picture's collision result.
#[test]
fn retained_pictures_restore_contacts_and_repaint_hidden_geometry_changes() {
    let mut video = Video::default();
    let mut picture = Picture {
        appearance: Appearance {
            foreground: 28,
            background: 28,
            walls_in_front: true,
            player_width: 4,
            ..Appearance::default()
        },
        walls: [PlayfieldRow::new(0xf0, 0xff, 0xff); HEIGHT],
        player_visible: [true; HEIGHT],
        objects: [Object {
            line: ObjectLine {
                color: 28,
                ..ObjectLine::default()
            },
            rows: [0xff; HEIGHT],
        }; 2],
        ..Picture::default()
    };
    video.paint(&picture);
    let expected = video.contacts;
    let pixels = video.frame.pixels.clone();
    assert!(expected.objects_touch && expected.player_walls);
    video.contacts = Contacts::default();
    video.paint(&picture);
    assert_eq!(video.contacts, expected);
    assert_eq!(video.frame.painted_lines, HEIGHT);

    picture.objects[0].rows.fill(0);
    video.paint(&picture);
    assert_eq!(video.frame.pixels, pixels);
    assert!(!video.contacts.objects_touch);
    assert!(!video.contacts.player_objects[0]);
    picture.appearance.foreground = 72;
    video.paint(&picture);
    assert_eq!(video.frame.pixels, vec![72; WIDTH * HEIGHT]);
    video.paint(&Picture::default());
    assert_eq!(video.contacts, Contacts::default());
    assert!(video.frame.pixels.iter().all(|pixel| *pixel == 0));
}
