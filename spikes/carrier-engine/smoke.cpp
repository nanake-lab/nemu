// Minimal hardware feasibility probe. Not the full P0-10 acceptance suite.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <tlhelp32.h>
#include "llama.h"
#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

namespace fs = std::filesystem;
using Clock = std::chrono::steady_clock;

void require_quiet() {
    const HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snapshot == INVALID_HANDLE_VALUE) throw std::runtime_error("Cannot inspect runner processes");
    PROCESSENTRY32W process{};
    process.dwSize = sizeof(process);
    if (!Process32FirstW(snapshot, &process)) {
        CloseHandle(snapshot);
        throw std::runtime_error("Cannot enumerate processes");
    }
    do {
        if (_wcsicmp(process.szExeFile, L"Runner.Listener.exe") == 0 ||
            _wcsicmp(process.szExeFile, L"Runner.Worker.exe") == 0) {
            CloseHandle(snapshot);
            throw std::runtime_error("Runner is online; GPU probe prohibited");
        }
    } while (Process32NextW(snapshot, &process));
    const auto error = GetLastError();
    CloseHandle(snapshot);
    if (error != ERROR_NO_MORE_FILES) throw std::runtime_error("Process enumeration incomplete");
}

template<class T> T symbol(HMODULE module, const char * name) {
    const auto value = GetProcAddress(module, name);
    if (!value) throw std::runtime_error(std::string("Missing API: ") + name);
    return reinterpret_cast<T>(value);
}
#define BIND(module, name) const auto name##_fn = symbol<decltype(&name)>(module, #name)

struct Sample {
    std::vector<float> logits;
    std::vector<llama_token> tokens;
};

float maximum_difference(const Sample & a, const Sample & b) {
    if (a.logits.size() != b.logits.size()) throw std::runtime_error("Logit size mismatch");
    float difference = 0;
    for (size_t i = 0; i < a.logits.size(); ++i) {
        difference = std::max(difference, std::abs(a.logits[i] - b.logits[i]));
    }
    return difference;
}

int main(int argc, char ** argv) {
    try {
        if (argc != 5 || std::string(argv[4]) != "--quiet-window-confirmed") {
            throw std::runtime_error("Usage: smoke ENGINE_DIR MODEL.gguf NEW_OUTPUT_DIR --quiet-window-confirmed");
        }
        require_quiet(); // Before loading any engine/backend DLL.
        const fs::path engine = fs::absolute(argv[1]);
        const fs::path output = fs::absolute(argv[3]);
        if (fs::exists(output)) throw std::runtime_error("Output already exists; preserve evidence");
        fs::create_directories(output);
        // Process-local loader configuration; no user/system settings are changed.
        if (!SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_DEFAULT_DIRS) ||
            !AddDllDirectory(engine.c_str())) throw std::runtime_error("Cannot configure DLL directory");
        const auto library = LoadLibraryExW((engine / "llama.dll").c_str(), nullptr,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
        if (!library) throw std::runtime_error("Cannot load pinned llama.dll");
        const auto ggml = GetModuleHandleW(L"ggml.dll");
        if (!ggml) throw std::runtime_error("ggml.dll not loaded");
        const auto ggml_base = GetModuleHandleW(L"ggml-base.dll");
        if (!ggml_base) throw std::runtime_error("ggml-base.dll not loaded");
        BIND(ggml, ggml_backend_load_all_from_path);
        BIND(ggml, ggml_backend_dev_count);
        BIND(ggml, ggml_backend_dev_get);
        BIND(ggml_base, ggml_backend_dev_type);
        BIND(ggml_base, ggml_backend_dev_name);
        BIND(library, llama_backend_init);
        BIND(library, llama_backend_free);
        BIND(library, llama_model_default_params);
        BIND(library, llama_context_default_params);
        BIND(library, llama_model_load_from_file);
        BIND(library, llama_model_free);
        BIND(library, llama_init_from_model);
        BIND(library, llama_free);
        BIND(library, llama_model_get_vocab);
        BIND(library, llama_model_n_embd);
        BIND(library, llama_model_n_layer);
        BIND(library, llama_vocab_n_tokens);
        BIND(library, llama_tokenize);
        BIND(library, llama_token_to_piece);
        BIND(library, llama_vocab_is_eog);
        BIND(library, llama_get_memory);
        BIND(library, llama_memory_clear);
        BIND(library, llama_set_adapter_cvec);
        BIND(library, llama_batch_get_one);
        BIND(library, llama_decode);
        BIND(library, llama_get_logits_ith);
        ggml_backend_load_all_from_path_fn(engine.string().c_str());
        bool cuda_found = false;
        for (size_t i = 0; i < ggml_backend_dev_count_fn(); ++i) {
            const auto device = ggml_backend_dev_get_fn(i);
            if (ggml_backend_dev_type_fn(device) == GGML_BACKEND_DEVICE_TYPE_GPU &&
                std::string(ggml_backend_dev_name_fn(device)).find("CUDA") != std::string::npos) {
                cuda_found = true;
            }
        }
        if (!cuda_found) throw std::runtime_error("CUDA backend absent; CPU fallback is not a GPU result");
        llama_backend_init_fn();
        auto mp = llama_model_default_params_fn();
        mp.n_gpu_layers = -1;
        auto * model = llama_model_load_from_file_fn(argv[2], mp);
        if (!model) throw std::runtime_error("Model load failed");
        const auto * vocab = llama_model_get_vocab_fn(model);
        const int embd = llama_model_n_embd_fn(model);
        const int layers = llama_model_n_layer_fn(model);
        const int vocabulary = llama_vocab_n_tokens_fn(vocab);
        if (embd != 2560 || layers != 36 || vocabulary != 151936) {
            throw std::runtime_error("Unexpected pinned model shape");
        }
        auto cp = llama_context_default_params_fn();
        cp.n_ctx = 256;
        cp.n_batch = 256;
        cp.n_ubatch = 256;
        cp.n_threads = 4;
        cp.n_threads_batch = 4;
        cp.flash_attn_type = LLAMA_FLASH_ATTN_TYPE_DISABLED;
        std::ofstream configuration(output / "configuration.tsv");
        configuration << "n_embd\tn_layer\tn_vocab\tn_ctx\tn_batch\tthreads\tn_gpu_layers\tvector_layer\n"
            << embd << '\t' << layers << '\t' << vocabulary << "\t256\t256\t4\t-1\t17\n";
        std::ofstream records(output / "requests.tsv");
        records << "language\tmode\tapi_called\tset_return\tprompt_tokens\tprefill_wall_ms\tgreedy_tokens\n";
        std::ofstream comparisons(output / "comparisons.tsv");
        comparisons << "language\tzero_vs_plain_maxabs\tclear_vs_plain_maxabs\tA_repeat_maxabs\tA_vs_fresh_maxabs\tA_vs_plain_maxabs\tB_vs_A_maxabs\tzero_tokens_equal\tclear_tokens_equal\n";
        const std::array<std::pair<std::string, std::string>, 3> prompts = {{
            {"zh", u8"用中文回复一句简短的问候。"},
            {"en", "Reply with a short greeting in English."},
            {"ja", u8"日本語で短い挨拶を一文だけ返してください。"}
        }};
        const auto started = Clock::now();
        for (const auto & entry : prompts) {
            auto * context = llama_init_from_model_fn(model, cp);
            if (!context) throw std::runtime_error("Context creation failed");
            const std::string prompt = "<|im_start|>user\n" + entry.second +
                "<|im_end|>\n<|im_start|>assistant\n";
            auto run = [&](llama_context * ctx, const std::string & mode, float amplitude, bool install) {
                require_quiet();
                if (std::chrono::duration<double>(Clock::now() - started).count() > 600) {
                    throw std::runtime_error("Minimal probe exceeded ten minutes");
                }
                llama_memory_clear_fn(llama_get_memory_fn(ctx), true);
                std::vector<float> vector(static_cast<size_t>(layers - 1) * embd, 0.0f);
                vector[static_cast<size_t>(17 - 1) * embd] = amplitude;
                int code = 0;
                const bool api_called = install || mode == "clear";
                if (api_called) {
                    code = llama_set_adapter_cvec_fn(ctx, install ? vector.data() : nullptr,
                        install ? vector.size() : 0, embd, 17, 17);
                    if (code != 0) throw std::runtime_error("Control-vector update failed");
                }
                std::vector<llama_token> input(256);
                const auto count = llama_tokenize_fn(vocab, prompt.data(), static_cast<int32_t>(prompt.size()),
                    input.data(), static_cast<int32_t>(input.size()), false, true);
                if (count <= 0 || count > 240) throw std::runtime_error("Prompt tokenization failed");
                const auto before = Clock::now();
                if (llama_decode_fn(ctx, llama_batch_get_one_fn(input.data(), count)) != 0) {
                    throw std::runtime_error("Prefill failed");
                }
                auto * logits = llama_get_logits_ith_fn(ctx, -1);
                if (!logits) throw std::runtime_error("Missing logits");
                Sample sample;
                sample.logits.assign(logits, logits + vocabulary);
                for (const auto value : sample.logits) {
                    if (!std::isfinite(value)) throw std::runtime_error("Non-finite logits");
                }
                const double elapsed = std::chrono::duration<double, std::milli>(Clock::now() - before).count();
                const auto name = entry.first + "-" + mode;
                std::ofstream raw(output / (name + ".f32"), std::ios::binary);
                raw.write(reinterpret_cast<const char *>(sample.logits.data()),
                    static_cast<std::streamsize>(sample.logits.size() * sizeof(float)));
                if (!raw) throw std::runtime_error("Cannot save raw logits");
                std::ofstream text(output / (name + ".txt"), std::ios::binary);
                for (int step = 0; step < 8; ++step) {
                    llama_token token = static_cast<llama_token>(std::max_element(logits, logits + vocabulary) - logits);
                    sample.tokens.push_back(token);
                    if (llama_vocab_is_eog_fn(vocab, token)) break;
                    char piece[1024];
                    const auto size = llama_token_to_piece_fn(vocab, token, piece, sizeof(piece), 0, true);
                    if (size < 0) throw std::runtime_error("Token piece exceeds buffer");
                    text.write(piece, size);
                    if (step == 7) break;
                    require_quiet();
                    if (llama_decode_fn(ctx, llama_batch_get_one_fn(&token, 1)) != 0) {
                        throw std::runtime_error("Decode failed");
                    }
                    logits = llama_get_logits_ith_fn(ctx, -1);
                    if (!logits) throw std::runtime_error("Missing decode logits");
                }
                records << entry.first << '\t' << mode << '\t' << api_called << '\t' << code
                    << '\t' << count << '\t' << elapsed << '\t';
                for (auto token : sample.tokens) records << token << ',';
                records << '\n'; records.flush();
                return sample;
            };
            const auto plain = run(context, "plain", 0, false);
            const auto zero = run(context, "zero", 0, true);
            const auto a = run(context, "A", 1, true);
            const auto b = run(context, "B", -1, true);
            const auto clear = run(context, "clear", 0, false);
            const auto again = run(context, "A-again", 1, true);
            auto * fresh = llama_init_from_model_fn(model, cp);
            if (!fresh) throw std::runtime_error("Fresh context failed");
            const auto fresh_a = run(fresh, "fresh-A", 1, true);
            comparisons << entry.first << '\t' << maximum_difference(zero, plain) << '\t'
                << maximum_difference(clear, plain) << '\t' << maximum_difference(again, a) << '\t'
                << maximum_difference(a, fresh_a) << '\t' << maximum_difference(a, plain) << '\t'
                << maximum_difference(b, a) << '\t' << (zero.tokens == plain.tokens) << '\t'
                << (clear.tokens == plain.tokens) << '\n'; comparisons.flush();
            llama_free_fn(fresh);
            llama_free_fn(context);
        }
        llama_model_free_fn(model);
        llama_backend_free_fn();
        std::ofstream(output / "execution-completed.txt") << "Execution completed; acceptance requires analysis.\n";
        return 0;
    } catch (const std::exception & error) {
        std::cerr << "smoke: " << error.what() << '\n';
        return 1;
    }
}
