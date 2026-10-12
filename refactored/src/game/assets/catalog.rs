use super::super::model::{
    ColorChoice, DragonId, GateId, ObjectDescriptor, PositionSource, RecordId, StateSource,
};
use super::artwork::{
    BAT_IMAGES, BLACK_GATE_POSITION, BRIDGE_IMAGES, CHALICE_IMAGES, DOT_IMAGES, DRAGON_IMAGES,
    EMPTY_IMAGES, GAME_NUMBER_IMAGES, GAME_NUMBER_POSITION, GATE_IMAGES, GOLD_GATE_POSITION,
    KEY_IMAGES, LIGHT_IMAGES, MAGNET_IMAGES, ROBINETT_CREDITS_IMAGES, ROBINETT_CREDITS_POSITION,
    SWORD_IMAGES, WHITE_GATE_POSITION,
};
use crate::video::SpriteWidth;

pub(in crate::game) static OBJECT_CATALOG: [ObjectDescriptor; 19] = [
    // Light.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Light),
        state: StateSource::Fixed(0),
        images: &LIGHT_IMAGES,
        color: ColorChoice::Fixed(0x28),
        monochrome: ColorChoice::Fixed(0x0c),
        style: SpriteWidth::Wide,
    },
    // GoldGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(GOLD_GATE_POSITION),
        state: StateSource::Gate(GateId::Gold),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // WhiteGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(WHITE_GATE_POSITION),
        state: StateSource::Gate(GateId::White),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // BlackGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(BLACK_GATE_POSITION),
        state: StateSource::Gate(GateId::Black),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Credits.
    ObjectDescriptor {
        position: PositionSource::Fixed(ROBINETT_CREDITS_POSITION),
        state: StateSource::Fixed(0),
        images: &ROBINETT_CREDITS_IMAGES,
        color: ColorChoice::Cycling,
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // GameNumber.
    ObjectDescriptor {
        position: PositionSource::Fixed(GAME_NUMBER_POSITION),
        state: StateSource::GameNumber,
        images: &GAME_NUMBER_IMAGES,
        color: ColorChoice::Fixed(0xc8),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Rhindle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Rhindle),
        state: StateSource::Dragon(DragonId::Rhindle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0e),
        style: SpriteWidth::Normal,
    },
    // Yorgle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Yorgle),
        state: StateSource::Dragon(DragonId::Yorgle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // Grundle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Grundle),
        state: StateSource::Dragon(DragonId::Grundle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0xc8),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Sword.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Sword),
        state: StateSource::Fixed(0),
        images: &SWORD_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // Bridge.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bridge),
        state: StateSource::Fixed(0),
        images: &BRIDGE_IMAGES,
        color: ColorChoice::Fixed(0x66),
        monochrome: ColorChoice::Fixed(0x02),
        style: SpriteWidth::Wide,
    },
    // GoldKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::GoldKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // WhiteKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::WhiteKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x0e),
        monochrome: ColorChoice::Fixed(0x0e),
        style: SpriteWidth::Normal,
    },
    // BlackKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::BlackKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Bat.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bat),
        state: StateSource::BatAnimation,
        images: &BAT_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Dot.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Dot),
        state: StateSource::Fixed(0),
        images: &DOT_IMAGES,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        style: SpriteWidth::Normal,
    },
    // Chalice.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Chalice),
        state: StateSource::Fixed(0),
        images: &CHALICE_IMAGES,
        color: ColorChoice::Cycling,
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // Magnet.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Magnet),
        state: StateSource::Fixed(0),
        images: &MAGNET_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // Empty.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bridge),
        state: StateSource::Fixed(0),
        images: &EMPTY_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
];
