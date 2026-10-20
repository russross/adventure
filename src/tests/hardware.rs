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
fn delta_uploads_reconstruct_complete_rgb_across_frame_changes() {
    let mut pixels = vec![0; WIDTH * HEIGHT];
    let mut output = TexturePixels::new(&[]).unwrap();
    output.bytes.fill(255);
    assert_eq!(output.convert_changed_rows(&pixels, None), [(0, HEIGHT)]);
    assert!(output.bytes.iter().all(|byte| *byte == 0));
    let previous = pixels.clone();
    assert!(
        output
            .convert_changed_rows(&pixels, Some(&previous))
            .is_empty()
    );

    // Adjacent changes coalesce; separated and final rows retain their own spans.
    for row in [0, 1, 15, HEIGHT - 1] {
        pixels[row * WIDTH + WIDTH - 1] = 28;
    }
    assert_eq!(
        output.convert_changed_rows(&pixels, Some(&previous)),
        [(0, 2), (15, 16), (HEIGHT - 1, HEIGHT)]
    );
    let expected: Vec<u8> = pixels.iter().flat_map(|color| rgb_color(*color)).collect();
    assert_eq!(output.bytes, expected);

    // A room-wide palette change and subsequent erasure replace all retained colors.
    let previous = pixels.clone();
    pixels.fill(72);
    assert_eq!(
        output.convert_changed_rows(&pixels, Some(&previous)),
        [(0, HEIGHT)]
    );
    assert_eq!(
        output.bytes,
        pixels
            .iter()
            .flat_map(|color| rgb_color(*color))
            .collect::<Vec<_>>()
    );
    let previous = pixels.clone();
    pixels.fill(0);
    output.convert_changed_rows(&pixels, Some(&previous));
    assert!(output.bytes.iter().all(|byte| *byte == 0));
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
        output.convert_changed_rows(&pixels, None);
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
        assert_eq!(hardware.indexed.as_deref(), Some(frame.pixels.as_slice()));
    }

    // Verify actual uploaded texture colors before presentation invalidates the backbuffer.
    frame.pixels.fill(72);
    frame.pixels[WIDTH + 159] = 28;
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
        .chunks_exact(4)
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
    frame.pixels.fill(28);
    hardware.present(&frame, &Sound::default()).unwrap();
    assert!(!hardware.crt.update(&hardware.display, &frame).unwrap());

    // Motion erases old pixels and changes neighboring light through the analog filters.
    frame.pixels[60 * WIDTH + 30..60 * WIDTH + 40].fill(72);
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
