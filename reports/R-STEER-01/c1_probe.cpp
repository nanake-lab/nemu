// C1: zero-injection measurement calibration only. No vector extraction or selection.
#include "../../spikes/carrier-engine/native_probe.hpp"
#include <iomanip>
#include <iostream>
#include <numeric>
#include <sstream>

std::string read_text(const fs::path & path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) throw std::runtime_error("Missing frozen input");
    return {std::istreambuf_iterator<char>(stream), std::istreambuf_iterator<char>()};
}

void save_ids(const fs::path & path, const std::vector<llama_token> & ids) {
    std::ofstream stream(path);
    for (const auto id : ids) stream << id << '\n';
    if (!stream) throw std::runtime_error("Cannot save token IDs");
}

int main(int argc, char ** argv) {
    try {
        if (argc != 5 || std::string(argv[4]) != "--quiet-window-confirmed") {
            throw std::runtime_error("Usage: c1_probe ENGINE MODEL RUN --quiet-window-confirmed");
        }
        const fs::path root = fs::absolute(argv[3]);
        Native api(fs::absolute(argv[1]), argv[2], 32, 248320, 15);
        const auto token_piece = symbol<decltype(&llama_token_to_piece)>(GetModuleHandleW(L"llama.dll"), "llama_token_to_piece");
        auto * ctx = api.llama_init_from_model(api.model, api.parameters());
        if (!ctx) throw std::runtime_error("Context unavailable");
        std::cout << "READY" << std::endl;
        std::string line;
        while (std::getline(std::cin, line) && line != "QUIT") {
            require_quiet();
            std::istringstream command(line);
            std::string name, mode, keep;
            if (!std::getline(command,name,'\t') || !std::getline(command,mode,'\t') || !std::getline(command,keep,'\t') ||
                name.empty() || name.find_first_not_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_") != std::string::npos ||
                (mode != "score" && mode != "free") || (keep != "full" && keep != "summary")) {
                throw std::runtime_error("Invalid local command");
            }
            const fs::path out = root / "requests" / name;
            if (!fs::create_directory(out)) throw std::runtime_error("Preserve existing evidence");
            const auto prompt = read_text(root / "prompts" / (name + ".txt"));
            auto prefix = api.tokenize(prompt);
            auto complete = api.tokenize(prompt + read_text(root / "continuations" / (name + ".txt")));
            if (prefix.size() > 448 || complete.size() < prefix.size() ||
                !std::equal(prefix.begin(),prefix.end(),complete.begin())) throw std::runtime_error("Unstable token boundary or context overflow");
            const size_t count = complete.size() - prefix.size();
            if (mode == "score" && (count == 0 || count > 112)) throw std::runtime_error("Continuation outside C1 budget");
            save_ids(out / "prompt-ids.tsv",prefix);
            save_ids(out / "complete-ids.tsv",complete);
            // Native hybrid memory_clear clears attention AND recurrent memory before full prefill.
            api.llama_memory_clear(api.llama_get_memory(ctx),true);
            if (api.control(ctx,nullptr,0) != 0) throw std::runtime_error("Cannot remove control vector");
            auto batch = api.llama_batch_get_one(prefix.data(),static_cast<int32_t>(prefix.size()));
            std::ofstream records(out / "logits.tsv");
            records << std::setprecision(17) << "step\ttarget\targmax\tlogZ\ttarget_logit\tlogp\teog\ttop12-id-logit\n";
            std::ofstream generated(out / "output.bin",std::ios::binary);
            const size_t limit = mode == "score" ? count : 64;
            llama_token next = 0;
            bool stopped = false;
            size_t steps = 0;
            double total = 0;
            for (size_t step = 0; step < limit; ++step) {
                require_quiet();
                if (api.llama_decode(ctx,batch) != 0) throw std::runtime_error("Decode failed");
                const auto * raw = api.llama_get_logits_ith(ctx,-1);
                if (!raw) throw std::runtime_error("Missing logits");
                std::vector<float> values(raw,raw + api.vocabulary);
                for (const float value : values) if (!std::isfinite(value)) throw std::runtime_error("Nonfinite logits");
                const auto best = static_cast<llama_token>(std::max_element(values.begin(),values.end()) - values.begin());
                double mass = 0;
                for (const float value : values) mass += std::exp(double(value) - values[best]);
                const double logz = double(values[best]) + std::log(mass);
                next = mode == "score" ? complete[prefix.size()+step] : best;
                const double logp = double(values[next]) - logz;
                total += logp;
                stopped = api.llama_vocab_is_eog(api.vocab,next);
                if (mode == "score" && stopped) throw std::runtime_error("Unexpected EOG in scored content");
                std::vector<int> indices(values.size());
                std::iota(indices.begin(),indices.end(),0);
                std::partial_sort(indices.begin(),indices.begin()+12,indices.end(),[&](int a,int b) {
                    return values[a] == values[b] ? a < b : values[a] > values[b];
                });
                records << step << '\t' << next << '\t' << best << '\t' << logz << '\t' << values[next] << '\t' << logp << '\t' << stopped << '\t';
                for (int i=0;i<12;++i) records << indices[i] << ':' << values[indices[i]] << ',';
                records << '\n'; records.flush();
                if (step == 0 && keep == "full") save_floats(out / "first-logits.f32",values);
                if (!stopped) {
                    std::vector<char> piece(512);
                    const int n = token_piece(api.vocab,next,piece.data(),static_cast<int32_t>(piece.size()),0,true);
                    if (n < 0) throw std::runtime_error("Token piece exceeds buffer");
                    generated.write(piece.data(),n);
                }
                ++steps;
                if (stopped) break;
                batch = api.llama_batch_get_one(&next,1);
            }
            if (!records || !generated) throw std::runtime_error("Cannot write evidence");
            std::ofstream summary(out / "completion.tsv");
            summary << std::setprecision(17) << "steps\teog\tlimit\tsum_logp\n" << steps << '\t' << stopped << '\t' << limit << '\t' << total << '\n';
            if (!summary) throw std::runtime_error("Cannot write completion");
            std::cout << "OK\t" << name << '\t' << steps << std::endl;
        }
        api.llama_free(ctx);
        return 0;
    } catch (const std::exception & error) {
        std::cerr << "c1_probe: " << error.what() << std::endl;
        return 1;
    }
}
