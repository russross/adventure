use super::super::model::{Bitmap, ImageChoice, Position, Room};

pub(in crate::game) static GATE_IMAGES: [ImageChoice; 13] = [
    ImageChoice {
        ceiling: 0x04,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 12,
        },
    },
    ImageChoice {
        ceiling: 0x08,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 10,
        },
    },
    ImageChoice {
        ceiling: 0x0c,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 8,
        },
    },
    ImageChoice {
        ceiling: 0x10,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 6,
        },
    },
    ImageChoice {
        ceiling: 0x14,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 4,
        },
    },
    ImageChoice {
        ceiling: 0x18,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 2,
        },
    },
    ImageChoice {
        ceiling: 0x1c,
        bitmap: Bitmap::new(&GATE_IMAGE),
    },
    ImageChoice {
        ceiling: 0x20,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 2,
        },
    },
    ImageChoice {
        ceiling: 0x24,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 4,
        },
    },
    ImageChoice {
        ceiling: 0x28,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 6,
        },
    },
    ImageChoice {
        ceiling: 0x2c,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 8,
        },
    },
    ImageChoice {
        ceiling: 0x30,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 10,
        },
    },
    ImageChoice {
        ceiling: 0xff,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 12,
        },
    },
];

pub(in crate::game) static GATE_IMAGE: [u8; 17] = [
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0x00, // |        |
];

pub(in crate::game) const GOLD_GATE_POSITION: Position = Position::new(Room::GoldCastle, 77, 49);

pub(in crate::game) const WHITE_GATE_POSITION: Position = Position::new(Room::WhiteCastle, 77, 49);

pub(in crate::game) const BLACK_GATE_POSITION: Position = Position::new(Room::BlackCastle, 77, 49);

pub(in crate::game) static LIGHT_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&LIGHT_IMAGE),
}];

pub(in crate::game) static LIGHT_IMAGE: [u8; 33] = [
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0x00, // |        |
];

pub(in crate::game) static BRIDGE_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&BRIDGE_IMAGE),
}];

pub(in crate::game) static BRIDGE_IMAGE: [u8; 25] = [
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0x00, // |        |
];

pub(in crate::game) static GAME_ONE_IMAGE: [u8; 8] = [
    0x04, // |     █  |
    0x0c, // |    ██  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x0e, // |    ███ |
    0x00, // |        |
];

pub(in crate::game) static KEY_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&KEY_IMAGE),
}];

pub(in crate::game) static KEY_IMAGE: [u8; 4] = [
    0x07, // |     ███|
    0xfd, // |██████ █|
    0xa7, // |█ █  ███|
    0x00, // |        |
];

pub(in crate::game) static GAME_TWO_IMAGE: [u8; 8] = [
    0x0e, // |    ███ |
    0x11, // |   █   █|
    0x01, // |       █|
    0x02, // |      █ |
    0x04, // |     █  |
    0x08, // |    █   |
    0x1f, // |   █████|
    0x00, // |        |
];

pub(in crate::game) static GAME_THREE_IMAGE: [u8; 8] = [
    0x0e, // |    ███ |
    0x11, // |   █   █|
    0x01, // |       █|
    0x06, // |     ██ |
    0x01, // |       █|
    0x11, // |   █   █|
    0x0e, // |    ███ |
    0x00, // |        |
];

pub(in crate::game) static BAT_IMAGES: [ImageChoice; 2] = [
    ImageChoice {
        ceiling: 0x03,
        bitmap: Bitmap::new(&BAT_FRAME_ONE),
    },
    ImageChoice {
        ceiling: 0xff,
        bitmap: Bitmap::new(&BAT_FRAME_TWO),
    },
];

pub(in crate::game) static BAT_FRAME_ONE: [u8; 8] = [
    0x81, // |█      █|
    0x81, // |█      █|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xff, // |████████|
    0x5a, // | █ ██ █ |
    0x66, // | ██  ██ |
    0x00, // |        |
];

pub(in crate::game) static BAT_FRAME_TWO: [u8; 12] = [
    0x01, // |       █|
    0x80, // |█       |
    0x01, // |       █|
    0x80, // |█       |
    0x3c, // |  ████  |
    0x5a, // | █ ██ █ |
    0x66, // | ██  ██ |
    0xc3, // |██    ██|
    0x81, // |█      █|
    0x81, // |█      █|
    0x81, // |█      █|
    0x00, // |        |
];

pub(in crate::game) static DRAGON_IMAGES: [ImageChoice; 4] = [
    ImageChoice {
        ceiling: 0x00,
        bitmap: Bitmap::new(&DRAGON_WALKING_IMAGE),
    },
    ImageChoice {
        ceiling: 0x01,
        bitmap: Bitmap::new(&DRAGON_DEAD_IMAGE),
    },
    ImageChoice {
        ceiling: 0x02,
        bitmap: Bitmap::new(&DRAGON_WALKING_IMAGE),
    },
    ImageChoice {
        ceiling: 0xff,
        bitmap: Bitmap::new(&DRAGON_BITING_IMAGE),
    },
];

pub(in crate::game) static DRAGON_WALKING_IMAGE: [u8; 21] = [
    0x06, // |     ██ |
    0x0f, // |    ████|
    0xf3, // |████  ██|
    0xfe, // |███████ |
    0x0e, // |    ███ |
    0x04, // |     █  |
    0x04, // |     █  |
    0x1e, // |   ████ |
    0x3f, // |  ██████|
    0x7f, // | ███████|
    0xe3, // |███   ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc7, // |██   ███|
    0xff, // |████████|
    0x3c, // |  ████  |
    0x08, // |    █   |
    0x8f, // |█   ████|
    0xe1, // |███    █|
    0x3f, // |  ██████|
    0x00, // |        |
];

pub(in crate::game) static DRAGON_BITING_IMAGE: [u8; 23] = [
    0x80, // |█       |
    0x40, // | █      |
    0x26, // |  █  ██ |
    0x1f, // |   █████|
    0x0b, // |    █ ██|
    0x0e, // |    ███ |
    0x1e, // |   ████ |
    0x24, // |  █  █  |
    0x44, // | █   █  |
    0x8e, // |█   ███ |
    0x1e, // |   ████ |
    0x3f, // |  ██████|
    0x7f, // | ███████|
    0x7f, // | ███████|
    0x7f, // | ███████|
    0x7f, // | ███████|
    0x3e, // |  █████ |
    0x1c, // |   ███  |
    0x08, // |    █   |
    0xf8, // |█████   |
    0x80, // |█       |
    0xe0, // |███     |
    0x00, // |        |
];

pub(in crate::game) static DRAGON_DEAD_IMAGE: [u8; 18] = [
    0x0c, // |    ██  |
    0x0c, // |    ██  |
    0x0c, // |    ██  |
    0x0e, // |    ███ |
    0x1b, // |   ██ ██|
    0x7f, // | ███████|
    0xce, // |██  ███ |
    0x80, // |█       |
    0xfc, // |██████  |
    0xfe, // |███████ |
    0xfe, // |███████ |
    0x7e, // | ██████ |
    0x78, // | ████   |
    0x20, // |  █     |
    0x6e, // | ██ ███ |
    0x42, // | █    █ |
    0x7e, // | ██████ |
    0x00, // |        |
];

pub(in crate::game) static SWORD_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&SWORD_IMAGE),
}];

pub(in crate::game) static SWORD_IMAGE: [u8; 6] = [
    0x20, // |  █     |
    0x40, // | █      |
    0xff, // |████████|
    0x40, // | █      |
    0x20, // |  █     |
    0x00, // |        |
];

pub(in crate::game) static DOT_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&DOT_IMAGE),
}];

pub(in crate::game) static DOT_IMAGE: [u8; 2] = [
    0x80, // |█       |
    0x00, // |        |
];

pub(in crate::game) static ROBINETT_CREDITS_IMAGE: [u8; 96] = [
    0xf0, // |████    |
    0x80, // |█       |
    0x80, // |█       |
    0x80, // |█       |
    0xf4, // |████ █  |
    0x04, // |     █  |
    0x87, // |█    ███|
    0xe5, // |███  █ █|
    0x87, // |█    ███|
    0x80, // |█       |
    0x05, // |     █ █|
    0xe5, // |███  █ █|
    0xa7, // |█ █  ███|
    0xe1, // |███    █|
    0x87, // |█    ███|
    0xe0, // |███     |
    0x01, // |       █|
    0xe0, // |███     |
    0xa0, // |█ █     |
    0xf0, // |████    |
    0x01, // |       █|
    0x40, // | █      |
    0xe0, // |███     |
    0x40, // | █      |
    0x40, // | █      |
    0x40, // | █      |
    0x01, // |       █|
    0xe0, // |███     |
    0xa0, // |█ █     |
    0xe0, // |███     |
    0x80, // |█       |
    0xe0, // |███     |
    0x01, // |       █|
    0x20, // |  █     |
    0x20, // |  █     |
    0xe0, // |███     |
    0xa0, // |█ █     |
    0xe0, // |███     |
    0x01, // |       █|
    0x01, // |       █|
    0x01, // |       █|
    0x88, // |█   █   |
    0xa8, // |█ █ █   |
    0xa8, // |█ █ █   |
    0xa8, // |█ █ █   |
    0xf8, // |█████   |
    0x01, // |       █|
    0xe0, // |███     |
    0xa0, // |█ █     |
    0xf0, // |████    |
    0x01, // |       █|
    0x80, // |█       |
    0xe0, // |███     |
    0x8f, // |█   ████|
    0x89, // |█   █  █|
    0x0f, // |    ████|
    0x8a, // |█   █ █ |
    0xe9, // |███ █  █|
    0x80, // |█       |
    0x8e, // |█   ███ |
    0x0a, // |    █ █ |
    0xee, // |███ ███ |
    0xa0, // |█ █     |
    0xe8, // |███ █   |
    0x88, // |█   █   |
    0xee, // |███ ███ |
    0x0a, // |    █ █ |
    0x8e, // |█   ███ |
    0xe0, // |███     |
    0xa4, // |█ █  █  |
    0xa4, // |█ █  █  |
    0x04, // |     █  |
    0x80, // |█       |
    0x08, // |    █   |
    0x0e, // |    ███ |
    0x0a, // |    █ █ |
    0x0a, // |    █ █ |
    0x80, // |█       |
    0x0e, // |    ███ |
    0x0a, // |    █ █ |
    0x0e, // |    ███ |
    0x08, // |    █   |
    0x0e, // |    ███ |
    0x80, // |█       |
    0x04, // |     █  |
    0x0e, // |    ███ |
    0x04, // |     █  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x80, // |█       |
    0x04, // |     █  |
    0x0e, // |    ███ |
    0x04, // |     █  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x00, // |        |
];

pub(in crate::game) const ROBINETT_CREDITS_POSITION: Position =
    Position::new(Room::RobinettCredits, 80, 105);

pub(in crate::game) static ROBINETT_CREDITS_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&ROBINETT_CREDITS_IMAGE),
}];

pub(in crate::game) static CHALICE_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&CHALICE_IMAGE),
}];

pub(in crate::game) static CHALICE_IMAGE: [u8; 10] = [
    0x81, // |█      █|
    0x81, // |█      █|
    0xc3, // |██    ██|
    0x7e, // | ██████ |
    0x7e, // | ██████ |
    0x3c, // |  ████  |
    0x18, // |   ██   |
    0x18, // |   ██   |
    0x7e, // | ██████ |
    0x00, // |        |
];

pub(in crate::game) static EMPTY_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&EMPTY_IMAGE),
}];

pub(in crate::game) static EMPTY_IMAGE: [u8; 1] = [
    0x00, // |        |
];

pub(in crate::game) const GAME_NUMBER_POSITION: Position =
    Position::new(Room::GameSelection, 80, 64);

pub(in crate::game) static GAME_NUMBER_IMAGES: [ImageChoice; 3] = [
    ImageChoice {
        ceiling: 0x01,
        bitmap: Bitmap::new(&GAME_ONE_IMAGE),
    },
    ImageChoice {
        ceiling: 0x03,
        bitmap: Bitmap::new(&GAME_TWO_IMAGE),
    },
    ImageChoice {
        ceiling: 0xff,
        bitmap: Bitmap::new(&GAME_THREE_IMAGE),
    },
];

pub(in crate::game) static MAGNET_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&MAGNET_IMAGE),
}];

pub(in crate::game) static MAGNET_IMAGE: [u8; 9] = [
    0x3c, // |  ████  |
    0x7e, // | ██████ |
    0xe7, // |███  ███|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0x00, // |        |
];
