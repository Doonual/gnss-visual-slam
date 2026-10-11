
// This only works on windows, the arboard crate has better cross-support
#define IMGUI_DISABLE_WIN32_DEFAULT_CLIPBOARD_FUNCTIONS

// Only use the latest non-obsolete functions
#define IMGUI_DISABLE_OBSOLETE_FUNCTIONS
#define IMGUI_DISABLE_OBSOLETE_KEYIO
// A Rust char is 32-bits, just do that
#define IMGUI_USE_WCHAR32

// Try to play thread-safe-ish. The variable definition is in wrapper.cpp
struct ImGuiContext;
extern thread_local ImGuiContext* MyImGuiTLS;
#define GImGui MyImGuiTLS
        