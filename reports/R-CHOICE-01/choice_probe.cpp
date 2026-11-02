// Bounded synthetic research; reuse the pinned, measured P0-10 injection API.
#include "../../spikes/carrier-engine/native_probe.hpp"
#include <array>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <sstream>

struct Capture {
    Native & api;
    std::vector<float> last;
    int count = 0;
    std::string error;
    explicit Capture(Native & a) : api(a) {}
    static bool callback(ggml_tensor * t, bool ask, void * user) noexcept {
        if (std::strcmp(t->name, "l_out-17")) return ask ? false : true;
        if (ask) return true;
        auto & c = *static_cast<Capture *>(user);
        try {
            if (++c.count != 1 || t->type != GGML_TYPE_F32 || t->ne[0] != c.api.embd ||
                t->ne[1] < 1 || t->ne[1] > 512 || t->ne[2] != 1 || t->ne[3] != 1 ||
                t->nb[0] != sizeof(float) || t->nb[1] != sizeof(float) * c.api.embd) {
                throw std::runtime_error("Unexpected extraction tensor");
            }
            c.last.resize(c.api.embd);
            c.api.ggml_backend_tensor_get(t, c.last.data(),
                static_cast<size_t>(t->ne[1] - 1) * t->nb[1], c.last.size() * sizeof(float));
            for (float v : c.last) if (!std::isfinite(v)) throw std::runtime_error("Nonfinite activation");
        } catch (const std::exception & e) { c.error = e.what(); return false; }
        return true;
    }
};

std::string read_text(const fs::path & path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) throw std::runtime_error("Cannot read input");
    return {std::istreambuf_iterator<char>(f), std::istreambuf_iterator<char>()};
}

std::vector<float> read_vector(const fs::path & path, int dimensions) {
    if (fs::file_size(path) != static_cast<size_t>(dimensions) * sizeof(float)) {
        throw std::runtime_error("Direction size mismatch");
    }
    std::vector<float> v(dimensions);
    std::ifstream f(path, std::ios::binary);
    f.read(reinterpret_cast<char *>(v.data()), static_cast<std::streamsize>(v.size() * sizeof(float)));
    if (!f) throw std::runtime_error("Cannot read direction");
    double norm2 = 0;
    for (float x : v) { if (!std::isfinite(x)) throw std::runtime_error("Invalid direction"); norm2 += x * double(x); }
    if (std::abs(std::sqrt(norm2) - 1) > 1e-6) throw std::runtime_error("Direction not unit length");
    return v;
}

int main(int argc, char ** argv) {
    try {
        if (argc != 5 || std::string(argv[4]) != "--quiet-window-confirmed") {
            throw std::runtime_error("Usage: choice_probe ENGINE MODEL RUN --quiet-window-confirmed");
        }
        const fs::path root = fs::absolute(argv[3]);
        Native api(fs::absolute(argv[1]), argv[2]);
        Capture capture(api);
        auto cp = api.parameters(); cp.cb_eval = Capture::callback; cp.cb_eval_user_data = &capture;
        auto * observed = api.llama_init_from_model(api.model, cp);
        auto * plain = api.llama_init_from_model(api.model, api.parameters());
        if (!observed || !plain) throw std::runtime_error("Cannot create contexts");
        std::array<llama_token, 3> labels{};
        for (int i = 0; i < 3; ++i) {
            const auto ids = api.tokenize(std::string(1, static_cast<char>('A' + i)));
            if (ids.size() != 1) throw std::runtime_error("Action label must be one token");
            labels[i] = ids[0];
        }
        std::cout << std::setprecision(17) << "RCHOICE READY\t" << labels[0] << '\t' << labels[1] << '\t' << labels[2] << std::endl;
        std::string line;
        while (std::getline(std::cin, line) && line != "QUIT") {
            require_quiet();
            std::istringstream in(line);
            std::array<std::string, 7> fields;
            for (auto & f : fields) if (!std::getline(in, f, '\t')) throw std::runtime_error("Bad local command");
            // mode, prompt basename, vector basename/none, amplitude, label count, activation output/none, logits output/none
            for (size_t i : {size_t(1), size_t(2), size_t(5), size_t(6)}) {
                if (fields[i].find_first_not_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.") != std::string::npos) {
                    throw std::runtime_error("Input/output must be a basename");
                }
            }
            const float amplitude = std::stof(fields[3]);
            const int nlabels = std::stoi(fields[4]);
            if ((nlabels != 2 && nlabels != 3) || !std::isfinite(amplitude) || amplitude < 0 || amplitude > 4) {
                throw std::runtime_error("Outside preregistered bounds");
            }
            auto * ctx = fields[0] == "P" ? plain : observed;
            api.llama_memory_clear(api.llama_get_memory(ctx), true);
            std::vector<float> vector;
            const std::vector<float> * direction = nullptr;
            if (fields[2] != "none") { vector = read_vector(root / "vectors" / fields[2], api.embd); direction = &vector; }
            if (api.control(ctx, direction, amplitude) != 0) throw std::runtime_error("Vector update failed");
            const auto prompt = read_text(root / "prompts" / fields[1]);
            auto tokens = api.tokenize("<|im_start|>user\n" + prompt + "<|im_end|>\n<|im_start|>assistant\n");
            capture.count = 0; capture.error.clear(); capture.last.clear();
            const int status = api.llama_decode(ctx, api.llama_batch_get_one(tokens.data(), static_cast<int32_t>(tokens.size())));
            if (status != 0 || !capture.error.empty() || (ctx == observed && capture.count != 1)) {
                throw std::runtime_error("Decode/extraction failed: " + capture.error);
            }
            const auto * logits = api.llama_get_logits_ith(ctx, -1);
            if (!logits) throw std::runtime_error("No logits");
            std::vector<float> values(logits, logits + api.vocabulary);
            for (float v : values) if (!std::isfinite(v)) throw std::runtime_error("Nonfinite logits");
            const auto argmax = static_cast<size_t>(std::max_element(values.begin(), values.end()) - values.begin());
            double denominator = 0, numerator = 0;
            for (float v : values) denominator += std::exp(double(v) - values[argmax]);
            for (int i = 0; i < nlabels; ++i) numerator += std::exp(double(values[labels[i]]) - values[argmax]);
            if (fields[5] != "none") {
                if (ctx != observed) throw std::runtime_error("No observation on plain context");
                save_floats(root / "activations" / fields[5], capture.last);
            }
            if (fields[6] != "none") save_floats(root / "checks" / fields[6], values);
            std::cout << "RCHOICE OK\t" << tokens.size() << '\t' << argmax << '\t'
                << values[labels[0]] << '\t' << values[labels[1]] << '\t' << values[labels[2]]
                << '\t' << numerator / denominator << '\t' << capture.count << std::endl;
        }
        api.llama_free(plain); api.llama_free(observed);
        return 0;
    } catch (const std::exception & e) {
        std::cerr << "choice_probe: " << e.what() << std::endl;
        return 1;
    }
}
