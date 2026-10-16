// The capture never serializes state. Fail explicitly if upstream starts using this interface.
#pragma once
#include "bspf.hxx"
#include <stdexcept>

class Serializer {
public:
    void putByte(uInt8) { throw std::logic_error("unexpected serialization"); }
    void putBool(bool) { throw std::logic_error("unexpected serialization"); }
    uInt8 getByte() { throw std::logic_error("unexpected deserialization"); }
    bool getBool() { throw std::logic_error("unexpected deserialization"); }
};

class Serializable {
public:
    virtual ~Serializable() = default;
    virtual bool save(Serializer&) const = 0;
    virtual bool load(Serializer&) = 0;
};
