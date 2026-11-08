// R-STEER-01 phase C0: zero modulation only. No direction extraction or selection.
#include "../../spikes/carrier-engine/native_probe.hpp"
#include <array>
#include <iomanip>
#include <iostream>
#include <numeric>
#include <sstream>

std::string read_text(const fs::path & path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) throw std::runtime_error("Missing frozen prompt");
    return {std::istreambuf_iterator<char>(stream), std::istreambuf_iterator<char>()};
}

int main(int argc, char ** argv) {
    try {
        if (argc != 5 || std::string(argv[4]) != "--quiet-window-confirmed") {
            throw std::runtime_error("Usage: calibration_probe ENGINE MODEL RUN --quiet-window-confirmed");
        }
        const fs::path root = fs::absolute(argv[3]);
        Native api(fs::absolute(argv[1]), argv[2], 32, 248320, 15);
        const auto token_piece = symbol<decltype(&llama_token_to_piece)>(
            GetModuleHandleW(L"llama.dll"), "llama_token_to_piece");
        auto * ctx = api.llama_init_from_model(api.model, api.parameters());
        if (!ctx) throw std::runtime_error("Context unavailable");
        std::array<llama_token, 4> codes{};
        for (size_t i = 0; i < codes.size(); ++i) {
            auto ids = api.tokenize(std::string(1, "ABXY"[i]));
            if (ids.size() != 1) throw std::runtime_error("Code must be one token");
            codes[i] = ids.front();
        }
        std::cout << "READY" << std::endl;
        std::string line;
        while (std::getline(std::cin, line) && line != "QUIT") {
            require_quiet();
            std::istringstream command(line);
            std::string name, length, keep;
            if (!std::getline(command, name, '\t') || !std::getline(command, length, '\t') ||
                !std::getline(command, keep, '\t') || name.empty() ||
                name.find_first_not_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_") != std::string::npos) {
                throw std::runtime_error("Invalid local command");
            }
            const int limit = std::stoi(length);
            if (limit != 1 && limit != 64) throw std::runtime_error("Outside phase C0 token budget");
            const fs::path output = root / "requests" / name;
            if (!fs::create_directory(output)) throw std::runtime_error("Preserve previous evidence");
            api.llama_memory_clear(api.llama_get_memory(ctx), true); // Both hybrid memories; P0-12.
            if (api.control(ctx, nullptr, 0) != 0) throw std::runtime_error("Cannot remove control vector");
            auto tokens = api.tokenize(read_text(root / "prompts" / (name + ".txt")));
            std::ofstream input(output / "input-ids.tsv");
            for (auto id : tokens) input << id << '\n';
            auto batch = api.llama_batch_get_one(tokens.data(), static_cast<int32_t>(tokens.size()));
            std::ofstream records(output / "logits.tsv");
            records << std::setprecision(17) << "step\targmax\tlogZ\tA\tB\tX\tY\teog\ttop12-id-logit\n";
            std::ofstream generated(output / "output.bin", std::ios::binary);
            llama_token next = 0;
            bool stopped = false;
            int steps = 0;
            for (int step = 0; step < limit; ++step) {
                require_quiet();
                if (api.llama_decode(ctx, batch) != 0) throw std::runtime_error("Decode failed");
                const auto * raw = api.llama_get_logits_ith(ctx, -1);
                if (!raw) throw std::runtime_error("Missing logits");
                std::vector<float> values(raw, raw + api.vocabulary);
                for (float value : values) if (!std::isfinite(value)) throw std::runtime_error("Nonfinite logits");
                next = static_cast<llama_token>(std::max_element(values.begin(), values.end()) - values.begin());
                double mass = 0;
                for (float value : values) mass += std::exp(double(value) - values[next]);
                const double logz = double(values[next]) + std::log(mass);
                std::vector<int> indices(values.size());
                std::iota(indices.begin(), indices.end(), 0);
                std::partial_sort(indices.begin(), indices.begin() + 12, indices.end(),
                    [&](int a, int b) { return values[a] == values[b] ? a < b : values[a] > values[b]; });
                stopped = api.llama_vocab_is_eog(api.vocab, next);
                records << step << '\t' << next << '\t' << logz;
                for (auto id : codes) records << '\t' << values[id];
                records << '\t' << stopped << '\t';
                for (int i = 0; i < 12; ++i) records << indices[i] << ':' << values[indices[i]] << ',';
                records << '\n'; records.flush();
                if (step == 0 && keep == "full") save_floats(output / "first-logits.f32", values);
                if (!stopped) {
                    std::vector<char> piece(256);
                    const int n = token_piece(api.vocab, next, piece.data(), static_cast<int32_t>(piece.size()), 0, true);
                    if (n < 0) throw std::runtime_error("Token piece exceeds evidence buffer");
                    generated.write(piece.data(), n);
                }
                ++steps;
                if (stopped) break;
                batch = api.llama_batch_get_one(&next, 1);
            }
            if (!generated || !records) throw std::runtime_error("Cannot write evidence");
            std::ofstream(output / "completion.tsv") << "steps\teog\tlimit\n" << steps << '\t' << stopped << '\t' << limit << '\n';
            std::cout << "OK\t" << name << '\t' << codes[0] << '\t' << codes[1] << '\t'
                << codes[2] << '\t' << codes[3] << std::endl;
        }
        api.llama_free(ctx);
        return 0;
    } catch (const std::exception & error) {
        std::cerr << "calibration_probe: " << error.what() << std::endl;
        return 1;
    }
}
