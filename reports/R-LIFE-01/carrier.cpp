// Research only: pinned P0-10 API, zero modulation, one unrestricted token slot.
#include "../../spikes/carrier-engine/native_probe.hpp"
#include <iomanip>
#include <iostream>

int main(int argc, char ** argv) {
    try {
        if (argc != 5 || std::string(argv[4]) != "--quiet-window-confirmed")
            throw std::runtime_error("Expected ENGINE MODEL RUN and quiet-window flag");
        const fs::path root = fs::absolute(argv[3]);
        Native api(fs::absolute(argv[1]), argv[2]);
        auto params = api.parameters();
        params.n_ctx = 8192; params.n_batch = 4096; params.n_ubatch = 512;
        auto * ctx = api.llama_init_from_model(api.model, params);
        if (!ctx) throw std::runtime_error("Cannot create context");
        const auto a = api.tokenize("A"), b = api.tokenize("B");
        if (a.size() != 1 || b.size() != 1) throw std::runtime_error("Labels must be one token");
        std::cout << "READY" << std::endl;
        std::string filename;
        while (std::getline(std::cin, filename) && filename != "QUIT") {
            require_quiet();
            if (filename.empty() || filename.find_first_not_of("abcdefghijklmnopqrstuvwxyz0123456789-.") != std::string::npos
                || filename == "." || filename == "..") throw std::runtime_error("Basename only");
            std::ifstream stream(root / filename, std::ios::binary);
            if (!stream) throw std::runtime_error("Missing prompt");
            const std::string prompt{std::istreambuf_iterator<char>(stream), std::istreambuf_iterator<char>()};
            const std::string text = "<|im_start|>user\n" + prompt + "<|im_end|>\n<|im_start|>assistant\n";
            std::vector<llama_token> tokens(4096);
            const auto n = api.llama_tokenize(api.vocab, text.data(), static_cast<int32_t>(text.size()),
                tokens.data(), static_cast<int32_t>(tokens.size()), false, true);
            if (n <= 0 || n > 4096) throw std::runtime_error("Prompt budget exceeded");
            tokens.resize(n);
            api.llama_memory_clear(api.llama_get_memory(ctx), true);
            if (api.control(ctx, nullptr, 0) != 0) throw std::runtime_error("Zero modulation reset failed");
            if (api.llama_decode(ctx, api.llama_batch_get_one(tokens.data(), n)) != 0)
                throw std::runtime_error("Prefill failed");
            const auto * logits = api.llama_get_logits_ith(ctx, -1);
            if (!logits) throw std::runtime_error("No logits");
            int argmax = 0;
            for (int i = 0; i < api.vocabulary; ++i) {
                if (!std::isfinite(logits[i])) throw std::runtime_error("Nonfinite logits");
                if (logits[i] > logits[argmax]) argmax = i;
            }
            double denominator = 0;
            for (int i = 0; i < api.vocabulary; ++i)
                denominator += std::exp((double(logits[i]) - logits[argmax]) / 0.7);
            const double pa = std::exp((double(logits[a[0]]) - logits[argmax]) / 0.7) / denominator;
            const double pb = std::exp((double(logits[b[0]]) - logits[argmax]) / 0.7) / denominator;
            std::cout << std::setprecision(17) << n << '\t' << pa << '\t' << pb << '\t'
                << argmax << '\t' << logits[a[0]] << '\t' << logits[b[0]] << '\t'
                << (argmax == a[0] ? "A" : argmax == b[0] ? "B" : "INVALID") << std::endl;
        }
        api.llama_free(ctx);
        return 0;
    } catch (const std::exception & error) {
        std::cerr << "rlife: " << error.what() << std::endl;
        return 1;
    }
}
