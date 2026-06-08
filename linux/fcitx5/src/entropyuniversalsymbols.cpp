// SPDX-License-Identifier: GPL-3.0-or-later
// Entropy Universal Symbols backend for Fcitx5.

#include <array>
#include <chrono>
#include <cstdlib>
#include <cstdint>
#include <fstream>
#include <memory>
#include <optional>
#include <sstream>
#include <string>
#include <sys/stat.h>
#include <ctime>
#include "fcitx-utils/handlertable.h"
#include "fcitx-utils/key.h"
#include "fcitx-utils/keysym.h"
#include "fcitx-utils/keysymgen.h"
#include "fcitx/addonfactory.h"
#include "fcitx/addoninstance.h"
#include "fcitx/addonmanager.h"
#include "fcitx/event.h"
#include "fcitx/inputcontext.h"
#include "fcitx/instance.h"

namespace fcitx {
namespace {

constexpr uint16_t KC_F13 = 0x0068;
constexpr uint16_t MOD_CTRL = 0x0100;
constexpr uint16_t MOD_SHIFT = 0x0200;
constexpr uint16_t MOD_ALT = 0x0400;

struct SmartSymbol {
    uint16_t trigger;
    const char *symbol;
};

std::string entropyCacheDir() {
    if (const char *xdgCacheHome = std::getenv("XDG_CACHE_HOME")) {
        return std::string(xdgCacheHome) + "/entropy";
    }
    if (const char *home = std::getenv("HOME")) {
        return std::string(home) + "/.cache/entropy";
    }
    return "/tmp/entropy";
}

void ensureDirectory(const std::string &dir) {
    if (dir.empty()) {
        return;
    }
    std::string current;
    for (char ch : dir) {
        current.push_back(ch);
        if (ch == '/' && current.size() > 1) {
            mkdir(current.c_str(), 0755);
        }
    }
    mkdir(current.c_str(), 0755);
}

void diagnosticLog(const std::string &message) {
    const auto dir = entropyCacheDir();
    ensureDirectory(dir);
    std::ofstream out(dir + "/fcitx5.log", std::ios::app);
    if (!out) {
        return;
    }

    const auto now = std::chrono::system_clock::now();
    const auto time = std::chrono::system_clock::to_time_t(now);
    char stamp[32] = {};
    if (std::strftime(stamp, sizeof(stamp), "%Y-%m-%dT%H:%M:%S",
                      std::localtime(&time))) {
        out << stamp << ' ';
    }
    out << message << '\n';
}

template <typename T> std::string hexValue(T value) {
    std::ostringstream out;
    out << "0x" << std::hex << static_cast<unsigned long long>(value);
    return out.str();
}

const std::array<SmartSymbol, 69> SMART_SYMBOLS{{
    // F13..F24
    {KC_F13, "{"},
    {uint16_t(KC_F13 + 1), "}"},
    {uint16_t(KC_F13 + 2), "["},
    {uint16_t(KC_F13 + 3), "]"},
    {uint16_t(KC_F13 + 4), "("},
    {uint16_t(KC_F13 + 5), ")"},
    {uint16_t(KC_F13 + 6), "<"},
    {uint16_t(KC_F13 + 7), ">"},
    {uint16_t(KC_F13 + 8), "#"},
    {uint16_t(KC_F13 + 9), "@"},
    {uint16_t(KC_F13 + 10), "№"},
    {uint16_t(KC_F13 + 11), "₽"},

    // Shift+F13..F24
    {uint16_t(MOD_SHIFT | KC_F13), "!"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 1)), "\""},
    {uint16_t(MOD_SHIFT | (KC_F13 + 2)), "$"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 3)), "%"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 4)), "&"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 5)), "'"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 6)), "*"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 7)), "+"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 8)), "="},
    {uint16_t(MOD_SHIFT | (KC_F13 + 9)), "?"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 10)), "|"},
    {uint16_t(MOD_SHIFT | (KC_F13 + 11)), "\\"},

    // Ctrl+F13..F24
    {uint16_t(MOD_CTRL | KC_F13), "«"},
    {uint16_t(MOD_CTRL | (KC_F13 + 1)), "»"},
    {uint16_t(MOD_CTRL | (KC_F13 + 2)), "€"},
    {uint16_t(MOD_CTRL | (KC_F13 + 3)), "—"},
    {uint16_t(MOD_CTRL | (KC_F13 + 4)), "–"},
    {uint16_t(MOD_CTRL | (KC_F13 + 5)), "•"},
    {uint16_t(MOD_CTRL | (KC_F13 + 6)), "×"},
    {uint16_t(MOD_CTRL | (KC_F13 + 7)), "±"},
    {uint16_t(MOD_CTRL | (KC_F13 + 8)), "≠"},
    {uint16_t(MOD_CTRL | (KC_F13 + 9)), "≈"},
    {uint16_t(MOD_CTRL | (KC_F13 + 10)), "✓"},
    {uint16_t(MOD_CTRL | (KC_F13 + 11)), "§"},

    // Alt+F13..F24
    {uint16_t(MOD_ALT | KC_F13), "."},
    {uint16_t(MOD_ALT | (KC_F13 + 1)), ","},
    {uint16_t(MOD_ALT | (KC_F13 + 2)), ";"},
    {uint16_t(MOD_ALT | (KC_F13 + 3)), ":"},
    {uint16_t(MOD_ALT | (KC_F13 + 4)), "/"},
    {uint16_t(MOD_ALT | (KC_F13 + 5)), "`"},
    {uint16_t(MOD_ALT | (KC_F13 + 6)), "^"},

    // Ctrl+Alt+F13..F19
    {uint16_t(MOD_CTRL | MOD_ALT | KC_F13), "б"},
    {uint16_t(MOD_CTRL | MOD_ALT | (KC_F13 + 1)), "ю"},
    {uint16_t(MOD_CTRL | MOD_ALT | (KC_F13 + 2)), "ж"},
    {uint16_t(MOD_CTRL | MOD_ALT | (KC_F13 + 3)), "э"},
    {uint16_t(MOD_CTRL | MOD_ALT | (KC_F13 + 4)), "х"},
    {uint16_t(MOD_CTRL | MOD_ALT | (KC_F13 + 5)), "ъ"},
    {uint16_t(MOD_CTRL | MOD_ALT | (KC_F13 + 6)), "ё"},

    // Ctrl+Alt+Shift+F13..F19
    {uint16_t(MOD_CTRL | MOD_ALT | MOD_SHIFT | KC_F13), "Б"},
    {uint16_t(MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 1)), "Ю"},
    {uint16_t(MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 2)), "Ж"},
    {uint16_t(MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 3)), "Э"},
    {uint16_t(MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 4)), "Х"},
    {uint16_t(MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 5)), "Ъ"},
    {uint16_t(MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 6)), "Ё"},

    // Ctrl+Shift+F13..F24
    {uint16_t(MOD_CTRL | MOD_SHIFT | KC_F13), "°"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 1)), "‰"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 2)), "′"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 3)), "″"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 4)), "‘"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 5)), "’"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 6)), "„"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 7)), "“"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 8)), "”"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 9)), "™"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 10)), "~"},
    {uint16_t(MOD_CTRL | MOD_SHIFT | (KC_F13 + 11)), "_"},
}};

std::optional<uint16_t> baseKeycodeForSym(KeySym sym) {
    if (sym >= FcitxKey_F13 && sym <= FcitxKey_F24) {
        return uint16_t(KC_F13 + (sym - FcitxKey_F13));
    }
    return std::nullopt;
}

uint16_t transportModifiers(KeyStates states) {
    uint16_t modifiers = 0;
    if (states.test(KeyState::Ctrl)) {
        modifiers |= MOD_CTRL;
    }
    if (states.test(KeyState::Shift)) {
        modifiers |= MOD_SHIFT;
    }
    if (states.test(KeyState::Alt)) {
        modifiers |= MOD_ALT;
    }
    return modifiers;
}

std::optional<std::string> symbolForKey(const Key &key) {
    const auto base = baseKeycodeForSym(key.sym());
    if (!base) {
        return std::nullopt;
    }
    const uint16_t trigger = *base | transportModifiers(key.states());
    for (const auto &entry : SMART_SYMBOLS) {
        if (entry.trigger == trigger) {
            return std::string(entry.symbol);
        }
    }
    return std::nullopt;
}

} // namespace

class EntropyUniversalSymbols final : public AddonInstance {
public:
    explicit EntropyUniversalSymbols(Instance *instance) : instance_(instance) {
        eventHandler_ = instance_->watchEvent(
            EventType::InputContextKeyEvent, EventWatcherPhase::Default,
            [this](Event &event) { handleKeyEvent(event); });
        diagnosticLog("Fcitx5 addon loaded");
    }

private:
    void handleKeyEvent(Event &event) {
        auto &keyEvent = static_cast<KeyEvent &>(event);
        const auto symbol = symbolForKey(keyEvent.key());
        const auto base = baseKeycodeForSym(keyEvent.key().sym());
        if (base) {
            diagnosticLog(std::string("transport key sym=") +
                          hexValue(keyEvent.key().sym()) + " base=" +
                          hexValue(*base) + " release=" +
                          (keyEvent.isRelease() ? "true" : "false") +
                          " matched=" + (symbol ? "true" : "false"));
        }
        if (!symbol) {
            return;
        }

        // Swallow both press and release for handled transport chords, but
        // commit text only on press.
        if (!keyEvent.isRelease()) {
            keyEvent.inputContext()->commitString(*symbol);
            diagnosticLog(std::string("committed symbol ") + *symbol);
        }
        keyEvent.filterAndAccept();
    }

    Instance *instance_;
    std::unique_ptr<HandlerTableEntry<EventHandler>> eventHandler_;
};

class EntropyUniversalSymbolsFactory final : public AddonFactory {
    AddonInstance *create(AddonManager *manager) override {
        return new EntropyUniversalSymbols(manager->instance());
    }
};

} // namespace fcitx

FCITX_ADDON_FACTORY_V2(entropyuniversalsymbols,
                       fcitx::EntropyUniversalSymbolsFactory);
