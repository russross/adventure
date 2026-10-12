use super::super::model::{ObjectId, Reaction, RecordId, Room};

pub(in crate::game) static RHINDLE_REACTIONS: [Reaction; 4] = [
    Reaction {
        from: RecordId::Sword,
        to: RecordId::Rhindle,
    },
    Reaction {
        from: RecordId::Rhindle,
        to: RecordId::Player,
    },
    Reaction {
        from: RecordId::Rhindle,
        to: RecordId::Chalice,
    },
    Reaction {
        from: RecordId::Rhindle,
        to: RecordId::WhiteKey,
    },
];

pub(in crate::game) static YORGLE_REACTIONS: [Reaction; 4] = [
    Reaction {
        from: RecordId::Sword,
        to: RecordId::Yorgle,
    },
    Reaction {
        from: RecordId::GoldKey,
        to: RecordId::Yorgle,
    },
    Reaction {
        from: RecordId::Yorgle,
        to: RecordId::Player,
    },
    Reaction {
        from: RecordId::Yorgle,
        to: RecordId::Chalice,
    },
];

pub(in crate::game) static GRUNDLE_REACTIONS: [Reaction; 6] = [
    Reaction {
        from: RecordId::Sword,
        to: RecordId::Grundle,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::Player,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::Chalice,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::Bridge,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::BlackKey,
    },
];

pub(in crate::game) const DRAGON_BITE_DELAYS: [[u8; 2]; 3] =
    [[0xd0, 0xe8], [0xf0, 0xf6], [0xf0, 0xf6]];

pub(in crate::game) static BAT_REACTIONS: [Reaction; 10] = [
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Chalice,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Sword,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Bridge,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::GoldKey,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::WhiteKey,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::BlackKey,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Rhindle,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Yorgle,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Grundle,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Magnet,
    },
];

pub(in crate::game) const GATE_OBJECTS: [ObjectId; 3] =
    [ObjectId::GoldGate, ObjectId::WhiteGate, ObjectId::BlackGate];

pub(in crate::game) const GATE_KEYS: [ObjectId; 3] =
    [ObjectId::GoldKey, ObjectId::WhiteKey, ObjectId::BlackKey];

pub(in crate::game) const CASTLE_INTERIORS: [Room; 3] = [
    Room::GoldTreasury,
    Room::WhiteCastleInterior,
    Room::BlackCastleInterior,
];

pub(in crate::game) const CASTLE_OUTSIDES: [Room; 3] =
    [Room::GoldCastle, Room::WhiteCastle, Room::BlackCastle];

pub(in crate::game) static MAGNET_REACTIONS: [Reaction; 6] = [
    Reaction {
        from: RecordId::GoldKey,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::WhiteKey,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::BlackKey,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::Sword,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::Bridge,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::Chalice,
        to: RecordId::Magnet,
    },
];
