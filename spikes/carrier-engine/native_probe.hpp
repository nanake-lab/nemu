// Pinned-header bindings for the experimental native injection probe only.
#pragma once
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <tlhelp32.h>
#include "llama.h"
#include <algorithm>
#include <cmath>
#include <filesystem>
#include <fstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace fs = std::filesystem;

inline void require_quiet() {
    const HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snapshot == INVALID_HANDLE_VALUE) throw std::runtime_error("Cannot inspect runner");
    PROCESSENTRY32W p{};
    p.dwSize = sizeof(p);
    if (!Process32FirstW(snapshot, &p)) {
        CloseHandle(snapshot);
        throw std::runtime_error("Cannot enumerate runner processes");
    }
    do {
        if (!_wcsicmp(p.szExeFile, L"Runner.Listener.exe") ||
            !_wcsicmp(p.szExeFile, L"Runner.Worker.exe") || !_wcsicmp(p.szExeFile, L"blender.exe")) {
            CloseHandle(snapshot);
            throw std::runtime_error("Quiet window absent");
        }
    } while (Process32NextW(snapshot, &p));
    const auto error = GetLastError();
    CloseHandle(snapshot);
    if (error != ERROR_NO_MORE_FILES) throw std::runtime_error("Incomplete process inventory");
}

template<class T> inline T symbol(HMODULE module, const char * name) {
    const auto address = GetProcAddress(module, name);
    if (!address) throw std::runtime_error(std::string("Missing pinned API: ") + name);
    return reinterpret_cast<T>(address);
}

inline void save_floats(const fs::path & path, const std::vector<float> & values) {
    std::ofstream stream(path, std::ios::binary);
    stream.write(reinterpret_cast<const char *>(values.data()),
        static_cast<std::streamsize>(values.size() * sizeof(float)));
    if (!stream) throw std::runtime_error("Cannot save float evidence");
}

struct Native {
#define DECLARE(name) decltype(&name) name = nullptr
    DECLARE(llama_backend_init); DECLARE(llama_backend_free);
    DECLARE(llama_model_default_params); DECLARE(llama_context_default_params);
    DECLARE(llama_model_load_from_file); DECLARE(llama_model_free);
    DECLARE(llama_init_from_model); DECLARE(llama_free);
    DECLARE(llama_model_get_vocab); DECLARE(llama_model_n_embd); DECLARE(llama_model_n_layer);
    DECLARE(llama_vocab_n_tokens); DECLARE(llama_vocab_is_eog); DECLARE(llama_tokenize);
    DECLARE(llama_get_memory); DECLARE(llama_memory_clear); DECLARE(llama_set_adapter_cvec);
    DECLARE(llama_decode); DECLARE(llama_batch_get_one); DECLARE(llama_get_logits_ith);
    DECLARE(ggml_backend_tensor_get); DECLARE(ggml_backend_tensor_set);
#undef DECLARE
    llama_model * model = nullptr;
    const llama_vocab * vocab = nullptr;
    int embd = 0, layers = 0, vocabulary = 0;

    Native(const fs::path & engine, const char * file) {
        require_quiet(); // Before backend loading, not merely before inference.
        if (!SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_DEFAULT_DIRS) ||
            !AddDllDirectory(engine.c_str())) throw std::runtime_error("DLL search setup failed");
        const auto library = LoadLibraryExW((engine / "llama.dll").c_str(), nullptr,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
        if (!library) throw std::runtime_error("Cannot load pinned llama.dll");
        const auto ggml = GetModuleHandleW(L"ggml.dll");
        const auto base = GetModuleHandleW(L"ggml-base.dll");
        if (!ggml || !base) throw std::runtime_error("GGML bindings missing");
        symbol<decltype(&ggml_backend_load_all_from_path)>(ggml, "ggml_backend_load_all_from_path")(engine.string().c_str());
        const auto count = symbol<decltype(&ggml_backend_dev_count)>(ggml, "ggml_backend_dev_count");
        const auto get = symbol<decltype(&ggml_backend_dev_get)>(ggml, "ggml_backend_dev_get");
        const auto type = symbol<decltype(&ggml_backend_dev_type)>(base, "ggml_backend_dev_type");
        const auto name = symbol<decltype(&ggml_backend_dev_name)>(base, "ggml_backend_dev_name");
        bool cuda = false;
        for (size_t i = 0; i < count(); ++i) {
            cuda |= type(get(i)) == GGML_BACKEND_DEVICE_TYPE_GPU &&
                std::string(name(get(i))).find("CUDA") != std::string::npos;
        }
        if (!cuda) throw std::runtime_error("Reject CPU fallback");
#define BIND(name) name = symbol<decltype(name)>(library, #name)
        BIND(llama_backend_init); BIND(llama_backend_free);
        BIND(llama_model_default_params); BIND(llama_context_default_params);
        BIND(llama_model_load_from_file); BIND(llama_model_free);
        BIND(llama_init_from_model); BIND(llama_free);
        BIND(llama_model_get_vocab); BIND(llama_model_n_embd); BIND(llama_model_n_layer);
        BIND(llama_vocab_n_tokens); BIND(llama_vocab_is_eog); BIND(llama_tokenize);
        BIND(llama_get_memory); BIND(llama_memory_clear); BIND(llama_set_adapter_cvec);
        BIND(llama_decode); BIND(llama_batch_get_one); BIND(llama_get_logits_ith);
#undef BIND
        ggml_backend_tensor_get = symbol<decltype(ggml_backend_tensor_get)>(base, "ggml_backend_tensor_get");
        ggml_backend_tensor_set = symbol<decltype(ggml_backend_tensor_set)>(base, "ggml_backend_tensor_set");
        llama_backend_init();
        auto mp = llama_model_default_params();
        mp.n_gpu_layers = -1;
        model = llama_model_load_from_file(file, mp);
        if (!model) throw std::runtime_error("Model load failed");
        vocab = llama_model_get_vocab(model);
        embd = llama_model_n_embd(model); layers = llama_model_n_layer(model);
        vocabulary = llama_vocab_n_tokens(vocab);
        if (embd != 2560 || layers != 36 || vocabulary != 151936) throw std::runtime_error("Model shape changed");
    }
    ~Native() { if (model) { llama_model_free(model); llama_backend_free(); } }
    Native(const Native &) = delete;
    Native & operator=(const Native &) = delete;

    llama_context_params parameters() const {
        auto cp = llama_context_default_params();
        cp.n_ctx = 512; cp.n_batch = 512; cp.n_ubatch = 512;
        cp.n_threads = 4; cp.n_threads_batch = 4;
        cp.flash_attn_type = LLAMA_FLASH_ATTN_TYPE_DISABLED;
        return cp;
    }
    std::vector<llama_token> tokenize(const std::string & text) const {
        std::vector<llama_token> tokens(512);
        const auto n = llama_tokenize(vocab, text.data(), static_cast<int32_t>(text.size()), tokens.data(),
            static_cast<int32_t>(tokens.size()), false, true);
        if (n <= 0 || n > 460) throw std::runtime_error("Prompt exceeds registered size");
        tokens.resize(n);
        return tokens;
    }
    int control(llama_context * ctx, const std::vector<float> * direction, float amplitude) const {
        if (!direction) return llama_set_adapter_cvec(ctx, nullptr, 0, embd, 17, 17);
        std::vector<float> buffer(static_cast<size_t>(layers - 1) * embd, 0.0f);
        for (int i = 0; i < embd; ++i) buffer[static_cast<size_t>(16) * embd + i] = (*direction)[i] * amplitude;
        return llama_set_adapter_cvec(ctx, buffer.data(), buffer.size(), embd, 17, 17);
    }
};
