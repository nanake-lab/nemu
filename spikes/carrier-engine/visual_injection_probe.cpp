// Informational paired-position probe; not concept validation or production J.
#include "activation_trace.hpp"
#include <chrono>
#include <iomanip>
#include <utility>

using Clock = std::chrono::steady_clock;

struct Result { std::vector<llama_token> predictions; };

Result execute(Native & api, llama_context * ctx, Trace * trace, const fs::path & output,
    const std::string & prompt, const std::vector<llama_token> & teacher,
    const std::string & site, const std::vector<float> * direction, float amplitude,
    bool all_positions, int native_action, int limit = 24, bool clear_state = true) {
    require_quiet();
    fs::create_directories(output);
    if (clear_state) api.llama_memory_clear(api.llama_get_memory(ctx), true);
    int code = 0;
    if (native_action >= 0) {
        code = api.control(ctx, native_action == 1 ? direction : nullptr, amplitude);
        if (code != 0) throw std::runtime_error("Request-level vector update rejected");
    }
    if (trace) trace->begin(output, site, native_action == 1 ? nullptr : direction, amplitude, all_positions);
    auto tokens = api.tokenize(prompt);
    std::ofstream(output / "prompt.txt", std::ios::binary) << prompt;
    std::ofstream inputs(output / "input-token-ids.tsv");
    for (auto token : tokens) inputs << token << '\n';
    std::ofstream settings(output / "settings.tsv");
    settings << "site\tamplitude\tmask\tnative_action\tapi_return\tcallback\thistory\n" << site << '\t'
        << amplitude << '\t' << (all_positions ? "all" : "last") << '\t' << native_action << '\t'
        << code << '\t' << (trace != nullptr) << '\t' << (!clear_state ? "retained-KV-and-recurrent-diagnostic" :
            teacher.empty() ? "greedy" : "forced-base-trajectory") << '\n';
    std::ofstream records(output / "tokens.tsv");
    records << "step\tpredicted\tfed_token\twall_ms\teog\n";
    Result result;
    const size_t steps = teacher.empty() ? static_cast<size_t>(limit) : teacher.size();
    auto batch = api.llama_batch_get_one(tokens.data(), static_cast<int32_t>(tokens.size()));
    llama_token fed = 0;
    for (size_t step = 0; step < steps; ++step) {
        require_quiet();
        if (trace) { trace->counts = {}; trace->step = static_cast<int>(step); }
        const auto begin = Clock::now();
        const auto status = api.llama_decode(ctx, batch);
        if (trace) trace->verify();
        if (status != 0) throw std::runtime_error("Native decode failed");
        const auto * logits = api.llama_get_logits_ith(ctx, -1);
        if (!logits) throw std::runtime_error("Missing logits");
        std::vector<float> values(logits, logits + api.vocabulary);
        for (float value : values) if (!std::isfinite(value)) throw std::runtime_error("Nonfinite logits");
        save_floats(output / (std::to_string(step) + "-logits.f32"), values);
        const auto prediction = static_cast<llama_token>(std::max_element(values.begin(), values.end()) - values.begin());
        result.predictions.push_back(prediction);
        fed = teacher.empty() ? prediction : teacher[step];
        const bool eog = api.llama_vocab_is_eog(api.vocab, fed);
        const double ms = std::chrono::duration<double, std::milli>(Clock::now() - begin).count();
        records << step << '\t' << prediction << '\t' << fed << '\t' << ms << '\t' << eog << '\n';
        records.flush();
        if (teacher.empty() && eog) break;
        batch = api.llama_batch_get_one(&fed, 1);
    }
    std::ofstream(output / "execution-completed.txt") << "All registered steps executed.\n";
    return result;
}


std::string read_prompt(const fs::path & path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) throw std::runtime_error("Prepared prompt missing");
    return std::string(std::istreambuf_iterator<char>(stream), std::istreambuf_iterator<char>());
}

int main(int argc, char ** argv) {
    try {
        if (argc != 7 || std::string(argv[6]) != "--quiet-window-confirmed") {
            throw std::runtime_error("Usage: probe ENGINE MODEL NEW_OUTPUT INPUTS STAGE --quiet-window-confirmed");
        }
        const int stage = std::stoi(argv[5]);
        if (stage < 1 || stage > 3) throw std::runtime_error("Stage must be 1, 2, or 3");
        require_quiet();
        const fs::path output = fs::absolute(argv[3]);
        if (fs::exists(output)) throw std::runtime_error("Preserve previous measurements");
        fs::create_directories(output);
        Native api(fs::absolute(argv[1]), argv[2], 32, 248320, 15);
        Trace trace(api);
        trace.attention_node = "attn_residual-15";
        trace.full_node = "l_out-15";
        auto cp = api.parameters();
        cp.cb_eval = Trace::callback; cp.cb_eval_user_data = &trace;
        auto * observed = api.llama_init_from_model(api.model, cp);
        if (!observed) throw std::runtime_error("Observed context failed");
        std::vector<float> basis(api.embd, 0); basis[0] = 1;
        save_floats(output / "fixed-basis-e0.f32", basis);
        for (const std::string lang : {"zh", "en", "ja"}) {
            const auto prompt = read_prompt(fs::path(argv[4]) / ("injection-" + lang + ".txt"));
            auto * plain = api.llama_init_from_model(api.model, api.parameters());
            if (!plain) throw std::runtime_error("Reference context failed");
            const auto trajectory = execute(api, plain, nullptr, output / lang / "plain", prompt,
                {}, "none", nullptr, 0, false, -1).predictions;
            api.llama_free(plain);
            auto run = [&](const std::string & mode, const std::string & site,
                float amplitude, bool all, int native) {
                execute(api, observed, &trace, output / lang / mode, prompt, trajectory,
                    site, &basis, amplitude, all, native);
            };
            if (stage == 1) {
                run("observe", "none", 0, false, 0);
                run("attention-zero", "old", 0, false, 0);
                run("full-zero", "full", 0, false, 0);
                run("native-zero", "native", 0, true, 1);
                for (const std::string site : {"old", "full", "native"}) {
                    execute(api, observed, &trace, output / lang / (site + "-zero-greedy"), prompt,
                        {}, site, &basis, 0, site == "native", site == "native" ? 1 : 0);
                }
            } else if (stage == 2) {
                for (const std::string site : {"old", "full", "native"}) {
                    const int native = site == "native" ? 1 : 0;
                    run(site + "-A", site, 1, true, native);
                    run(site + "-B", site, -1, true, native);
                    run(site + "-zero-after-B", site, 0, true, native);
                    run(site + "-A-again", site, 1, true, native);
                    for (const auto & fresh : {std::pair<const char *, float>{"fresh-A", 1.0f}, {"fresh-B", -1.0f}}) {
                        auto * context = api.llama_init_from_model(api.model, cp);
                        if (!context) throw std::runtime_error("Fresh context failed");
                        execute(api, context, &trace, output / lang / (site + "-" + fresh.first), prompt,
                            trajectory, site, &basis, fresh.second, true, native);
                        api.llama_free(context);
                    }
                    execute(api, observed, &trace, output / lang / (site + "-diagnostic-seed-A"), prompt,
                        {}, site, &basis, 1, true, native, 1);
                    execute(api, observed, &trace, output / lang / (site + "-diagnostic-retained-B"), prompt,
                        {}, site, &basis, -1, true, native, 1, false);
                }
                execute(api, observed, &trace, output / lang / "native-clear", prompt, trajectory,
                    "native", nullptr, 0, true, 0);
            } else {
                run("attention-basis-last", "old", 1, false, 0);
                run("attention-basis-all", "old", 1, true, 0);
                run("full-basis-last", "full", 1, false, 0);
                run("full-basis-all", "full", 1, true, 0);
                run("native-basis-all", "native", 1, true, 1);
            }
        }
        api.llama_free(observed);
        std::ofstream(output / "execution-completed.txt") << "Measurements only; user judges each item.\n";
        return 0;
    } catch (const std::exception & error) {
        std::cerr << "visual_injection_probe: " << error.what() << '\n';
        return 1;
    }
}
