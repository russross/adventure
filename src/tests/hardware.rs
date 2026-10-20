use super::*;

#[test]
#[ignore = "requires an SDL video and audio driver"]
fn default_audio_streams_continuous_pcm_and_mute_controls_delivery() {
    let mut hardware = SdlHardware::new(false, None).unwrap();
    let frame = Frame::default();
    let mut sound = Sound::default();
    sound.voice(4, 7, 15);
    hardware.present(&frame, &sound).unwrap();
    let audio = &hardware.audio;
    assert_eq!(audio.device.status(), sdl2::audio::AudioStatus::Playing);

    // Muting clears delivery immediately; resuming retains the existing synthesis state.
    hardware.muted = true;
    hardware.present(&frame, &sound).unwrap();
    let audio = &hardware.audio;
    assert_eq!(audio.device.status(), sdl2::audio::AudioStatus::Paused);
    assert_eq!(audio.device.size(), 0);
    hardware.muted = false;
    hardware.present(&frame, &sound).unwrap();
    let audio = &hardware.audio;
    assert_eq!(audio.device.status(), sdl2::audio::AudioStatus::Playing);
}

#[test]
fn scene_damage_conversion_matches_complete_palette_output() {
    let mut video = crate::video::Video::default();
    let mut picture = crate::video::Picture::default();
    picture.objects[0].line.color = 72;
    picture.objects[0].rows[10] = 0xff;
    let mut output = TexturePixels::new(&[]).unwrap();
    let mut stamp = None;
    for step in 0..8 {
        match step {
            1 => picture.objects[0].line.x = 40,
            2 => picture.objects[0].rows[10] = 0,
            3 => picture.appearance.background = 8,
            4 => picture.objects[0].rows[HEIGHT - 1] = 0xff,
            5 => picture.objects[0].line.x = 159,
            6 => picture.appearance.background = 0,
            _ => {}
        }
        video.paint(&picture);
        output.convert_damage(&video.frame, stamp);
        stamp = Some(video.frame.stamp());
        let expected: Vec<u8> = video
            .frame
            .pixels()
            .iter()
            .flat_map(|color| rgb_color(*color))
            .collect();
        assert_eq!(output.bytes, expected, "step {step}");
    }
}

#[test]
fn packed_texture_formats_preserve_every_palette_color_and_snapshot() {
    let pixels: Vec<u8> = (0..WIDTH * HEIGHT).map(|index| index as u8).collect();
    let expected: Vec<u8> = pixels.iter().flat_map(|color| rgb_color(*color)).collect();
    for format in [
        PixelFormatEnum::ARGB8888,
        PixelFormatEnum::ABGR8888,
        PixelFormatEnum::RGBA8888,
        PixelFormatEnum::BGRA8888,
        PixelFormatEnum::RGB888,
        PixelFormatEnum::BGR888,
        PixelFormatEnum::RGB24,
    ] {
        let mut output = TexturePixels::new(&[PixelFormatEnum::YV12, format]).unwrap();
        assert_eq!(output.format, format);
        let mut frame = Frame::default();
        frame.pixels_mut().copy_from_slice(&pixels);
        output.convert_damage(&frame, None);
        let pitch = output.pitch() as u32;
        let surface = Surface::from_data(
            &mut output.bytes,
            WIDTH as u32,
            HEIGHT as u32,
            pitch,
            format,
        )
        .unwrap();
        let rgb = surface.convert_format(PixelFormatEnum::RGB24).unwrap();
        // SDL performs an independent format conversion, as snapshot readers do.
        assert_eq!(rgb.without_lock().unwrap(), expected);
    }
}

#[test]
#[ignore = "requires an SDL video and audio driver; run with dummy drivers in headless environments"]
fn sdl_idle_redraws_resets_uploads_and_frame_boundaries() {
    let mut hardware = SdlHardware::with_renderer(false, Some(12), RendererMode::Simple).unwrap();
    let sound = Sound::default();
    let mut frame = Frame::default();
    hardware.present(&frame, &sound).unwrap();
    hardware.sample_controls().unwrap();
    let initial = hardware.redraws;
    hardware.present(&frame, &sound).unwrap();
    let settled = hardware.redraws;
    hardware.present(&frame, &sound).unwrap();
    assert_eq!(hardware.redraws, settled);
    assert!(settled >= initial);
    assert_eq!(hardware.frames, 3);

    // Real SDL events force repainting even while the indexed picture is retained.
    let events = hardware._sdl.event().unwrap();
    for win_event in [
        WindowEvent::Exposed,
        WindowEvent::SizeChanged(640, 388),
        WindowEvent::Restored,
    ] {
        events
            .push_event(Event::Window {
                timestamp: 0,
                window_id: hardware.display.window().id(),
                win_event,
            })
            .unwrap();
        let redraws = hardware.redraws;
        hardware.present(&frame, &sound).unwrap();
        assert_eq!(hardware.redraws, redraws + 1);
    }
    for event in [
        Event::RenderTargetsReset { timestamp: 0 },
        Event::RenderDeviceReset { timestamp: 0 },
    ] {
        // The Rust binding cannot enqueue renderer-reset events; use the same observer.
        hardware.refresh.observe(&event);
        let redraws = hardware.redraws;
        hardware.present(&frame, &sound).unwrap();
        assert_eq!(hardware.redraws, redraws + 1);
        assert_eq!(hardware.stamp, Some(frame.stamp()));
    }

    // Verify actual uploaded texture colors before presentation invalidates the backbuffer.
    frame.pixels_mut().fill(72);
    frame.pixels_mut()[WIDTH + 159] = 28;
    hardware.present(&frame, &sound).unwrap();
    hardware
        .display
        .window_mut()
        .set_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .unwrap();
    hardware.sample_controls().unwrap();
    hardware
        .display
        .present_rgba(
            &hardware.pixels.bytes,
            WIDTH as u32,
            HEIGHT as u32,
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        )
        .unwrap();
    let rgba = hardware.display.capture_cpu().unwrap();
    let rgb: Vec<_> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| p[..3].iter().copied())
        .collect();
    let expected: Vec<u8> = frame
        .pixels
        .iter()
        .flat_map(|color| {
            let rgb = rgb_color(*color);
            [rgb, rgb].concat()
        })
        .collect();
    assert_eq!(rgb, expected);
    while hardware.frames < 12 {
        assert!(hardware.present(&frame, &sound).unwrap().is_some());
    }
    assert!(hardware.present(&frame, &sound).unwrap().is_none());
    assert_eq!(hardware.frames, 12);
}

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

#[test]
#[ignore = "requires an SDL video and audio driver"]
fn crt_retained_updates_match_fresh_render_after_resize_and_reset() {
    let mut hardware = SdlHardware::new(false, None).unwrap();
    hardware.display.window_mut().set_size(320, 194).unwrap();
    let mut frame = Frame::default();
    frame.pixels_mut().fill(28);
    hardware.present(&frame, &Sound::default()).unwrap();
    assert!(!hardware.crt.update(&hardware.display, &frame).unwrap());

    // Motion erases old pixels and changes neighboring light through the analog filters.
    frame.pixels_mut()[60 * WIDTH + 30..60 * WIDTH + 40].fill(72);
    hardware.crt.update(&hardware.display, &frame).unwrap();
    let mut fresh = CrtDisplay::new().unwrap();
    fresh.update(&hardware.display, &frame).unwrap();
    assert_eq!(
        hardware.crt.snapshot(&mut hardware.display).unwrap(),
        fresh.snapshot(&mut hardware.display).unwrap()
    );

    hardware.display.window_mut().set_size(640, 388).unwrap();
    hardware.sample_controls().unwrap();
    hardware.crt.update(&hardware.display, &frame).unwrap();
    fresh.update(&hardware.display, &frame).unwrap();
    assert_eq!(
        hardware.crt.snapshot(&mut hardware.display).unwrap(),
        fresh.snapshot(&mut hardware.display).unwrap()
    );
    hardware.display.invalidate();
    hardware.crt.draw(&mut hardware.display, true).unwrap();
    assert_eq!(
        hardware.crt.snapshot(&mut hardware.display).unwrap(),
        fresh.snapshot(&mut hardware.display).unwrap()
    );
}

#[test]
#[ignore = "requires an SDL video and audio driver"]
fn crt_skips_simple_preparation_and_switching_prepares_current_pixels() {
    let mut hardware = SdlHardware::new(false, None).unwrap();
    hardware.display.window_mut().set_size(320, 194).unwrap();
    let mut frame = Frame::default();
    frame.pixels_mut().fill(72);
    let unused_simple = hardware.pixels.bytes.clone();
    hardware.present(&frame, &Sound::default()).unwrap();
    assert_eq!(hardware.pixels.bytes, unused_simple);
    hardware.sample_controls().unwrap();
    hardware.present(&frame, &Sound::default()).unwrap();
    let redraws = hardware.redraws;
    hardware.present(&frame, &Sound::default()).unwrap();
    assert_eq!(hardware.redraws, redraws);

    // A real mode command must prepare the complete simple picture even when
    // its source is identical to the last CRT picture.
    hardware
        ._sdl
        .event()
        .unwrap()
        .push_event(Event::KeyDown {
            timestamp: 0,
            window_id: hardware.display.window().id(),
            keycode: Some(Keycode::C),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
        })
        .unwrap();
    hardware.present(&frame, &Sound::default()).unwrap();
    assert_eq!(hardware.renderer(), RendererMode::Simple);
    let mut expected = TexturePixels::new(&[PixelFormatEnum::RGBA32]).unwrap();
    expected.convert_damage(&frame, None);
    assert_eq!(hardware.pixels.bytes, expected.bytes);
}
