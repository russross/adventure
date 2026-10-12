use super::*;

#[test]
fn complete_lines_preserve_hidden_contacts_and_wide_seam_geometry() {
    let mut video = Video::default();
    let mut line = Scanline {
        appearance: Appearance {
            foreground: 28,
            walls_in_front: true,
            player_width: 4,
            mirrored: true,
            ..Appearance::default()
        },
        walls: PlayfieldRow::new(0x10, 0, 0),
        objects: [ObjectLine {
            x: 159,
            width: SpriteWidth::Wide,
            color: 72,
            bits: 0x81,
        }; 2],
        player_x: 159,
        player_visible: true,
    };
    video.begin_picture();
    video.paint(&line);
    assert_eq!(video.contacts.player_objects, [true, true]);
    assert!(video.contacts.objects_touch);
    assert_eq!(video.frame.pixels[159], 28);
    assert_eq!(video.frame.pixels[0], 28);
    line.player_visible = false;
    line.walls = PlayfieldRow::default();
    video.paint(&line);
    for x in [0, 1, 2, 27, 28, 29, 30, 159] {
        assert_eq!(video.frame.pixels[WIDTH + x], 72);
    }
    assert_eq!(video.frame.pixels[WIDTH + 158], 0);
    video.begin_picture();
    assert_eq!(video.contacts, Contacts::default());
}

#[test]
fn wall_cells_mirror_or_repeat_without_changing_the_line() {
    let mut video = Video::default();
    let mut line = Scanline {
        walls: PlayfieldRow::new(0x10, 0, 0),
        appearance: Appearance {
            foreground: 28,
            mirrored: true,
            ..Appearance::default()
        },
        ..Scanline::default()
    };
    video.paint(&line);
    assert_eq!(&video.frame.pixels[156..160], &[28; 4]);
    assert_eq!(video.frame.pixels[80], 0);
    line.appearance.mirrored = false;
    video.paint(&line);
    assert_eq!(&video.frame.pixels[WIDTH + 80..WIDTH + 84], &[28; 4]);
}
