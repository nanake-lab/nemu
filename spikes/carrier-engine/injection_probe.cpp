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
    bool all_positions, int native_action, int limit = 24, bool clear_kv = true) {
    require_quiet();
    fs::create_directories(output);
    if (clear_kv) api.llama_memory_clear(api.llama_get_memory(ctx), true);
    int code = 0;
    if (native_action >= 0) {
        code = api.control(ctx, native_action == 1 ? direction : nullptr, amplitude);
        if (code != 0) throw std::runtime_error("Request-level vector update rejected");
    }
    if (trace) trace->begin(output, site, native_action == 1 ? nullptr : direction, amplitude, all_positions);
    auto tokens = api.tokenize("<|im_start|>user\n" + prompt + "<|im_end|>\n<|im_start|>assistant\n");
    std::ofstream(output / "prompt.txt", std::ios::binary) << prompt;
    std::ofstream inputs(output / "input-token-ids.tsv");
    for (auto token : tokens) inputs << token << '\n';
    std::ofstream settings(output / "settings.tsv");
    settings << "site\tamplitude\tmask\tnative_action\tapi_return\tcallback\thistory\n" << site << '\t'
        << amplitude << '\t' << (all_positions ? "all" : "last") << '\t' << native_action << '\t'
        << code << '\t' << (trace != nullptr) << '\t' << (!clear_kv ? "INVALID-retained-KV-diagnostic" :
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

std::vector<float> normalize(const std::vector<double> & total) {
    double square = 0;
    for (double value : total) square += value * value;
    const double norm = std::sqrt(square);
    if (!std::isfinite(norm) || norm <= 0) throw std::runtime_error("Cannot normalize extracted contrast");
    std::vector<float> direction;
    for (double value : total) direction.push_back(static_cast<float>(value / norm));
    return direction;
}

int main(int argc, char ** argv) {
    try {
        if (argc != 5 || std::string(argv[4]) != "--quiet-window-confirmed") {
            throw std::runtime_error("Usage: injection_probe ENGINE MODEL NEW_OUTPUT --quiet-window-confirmed");
        }
        require_quiet();
        const fs::path output = fs::absolute(argv[3]);
        if (fs::exists(output)) throw std::runtime_error("Preserve previous measurements");
        fs::create_directories(output);
        Native api(fs::absolute(argv[1]), argv[2]);
        Trace trace(api);
        auto cp = api.parameters();
        cp.cb_eval = Trace::callback; cp.cb_eval_user_data = &trace;
        auto * observed = api.llama_init_from_model(api.model, cp);
        if (!observed) throw std::runtime_error("Observed context failed");
        // Exactly the three calm/excited calibration pairs from P0-07 direction_probe.py.
        // Extraction feasibility only: these are not four nature channels or new evidence of behavior.
        const std::array<std::pair<std::string, std::string>, 3> pairs = {{
            {u8"请平静地说：我们完成了今天的任务。", u8"请兴奋地说：我们完成了今天的任务。"},
            {"Say calmly: We finished today's task.", "Say excitedly: We finished today's task."},
            {u8"落ち着いて言ってください：今日の作業が終わりました。", u8"興奮して言ってください：今日の作業が終わりました。"}
        }};
        std::array<std::vector<double>, 2> total = {std::vector<double>(api.embd), std::vector<double>(api.embd)};
        for (size_t p = 0; p < pairs.size(); ++p) {
            std::array<std::vector<float>, 2> calm;
            for (int sign = 0; sign < 2; ++sign) {
                execute(api, observed, &trace, output / "extraction" / (std::to_string(p) + "-" + std::to_string(sign)),
                    sign == 0 ? pairs[p].first : pairs[p].second, {}, "none", nullptr, 0, false, 0, 1);
                if (sign == 0) calm = trace.last;
                else for (int site = 0; site < 2; ++site) for (int i = 0; i < api.embd; ++i) {
                    total[site][i] += (static_cast<double>(trace.last[site][i]) - calm[site][i]) / pairs.size();
                }
            }
        }
        const auto old_direction = normalize(total[0]);
        const auto full_direction = normalize(total[1]);
        save_floats(output / "direction-old.f32", old_direction);
        save_floats(output / "direction-full.f32", full_direction);
        std::vector<float> basis(api.embd, 0); basis[0] = 1;
        save_floats(output / "direction-basis.f32", basis);
        const std::array<std::pair<std::string, std::string>, 3> prompts = {{
            {"zh", u8"请逐字输出下面这句话，不添加其他内容：今天的努力值得肯定请带着信心继续向前走吧。"},
            {"en", "Repeat exactly, adding nothing: Today's effort matters. Keep going with confidence."},
            {"ja", u8"次の文だけをそのまま出力してください：今日の努力には価値があります。自信を持って進みましょう。"}
        }};
        const auto started = Clock::now();
        for (const auto & prompt : prompts) {
            auto * plain = api.llama_init_from_model(api.model, api.parameters());
            if (!plain) throw std::runtime_error("Reference context failed");
            const auto trajectory = execute(api, plain, nullptr, output / prompt.first / "plain", prompt.second,
                {}, "none", nullptr, 0, false, -1).predictions;
            api.llama_free(plain);
            auto run = [&](const std::string & mode, const std::string & site, const std::vector<float> * dir,
                float amplitude, bool all = false, int native = 0) {
                if (std::chrono::duration<double>(Clock::now() - started).count() > 900) {
                    throw std::runtime_error("Registered injection window exceeded fifteen minutes");
                }
                execute(api, observed, &trace, output / prompt.first / mode, prompt.second, trajectory,
                    site, dir, amplitude, all, native);
            };
            run("observe", "none", nullptr, 0);
            run("old-zero", "old", &basis, 0);
            run("full-zero", "full", &basis, 0);
            run("native-zero", "native", &basis, 0, true, 1);
            run("old-basis-last", "old", &basis, 1);
            run("full-basis-last", "full", &basis, 1);
            run("full-basis-all", "full", &basis, 1, true);
            run("native-basis-all", "native", &basis, 1, true, 1);
            run("old-reextracted-last", "old", &old_direction, 1);
            run("full-reextracted-last", "full", &full_direction, 1);
            // All updates use the same library context; KV is explicitly cleared before every request.
            run("native-A", "native", &full_direction, 1, true, 1);
            run("native-B", "native", &full_direction, -1, true, 1);
            run("native-zero-after-B", "native", &full_direction, 0, true, 1);
            run("native-A-again", "native", &full_direction, 1, true, 1);
            run("native-clear", "native", nullptr, 0, true, 0);
            // Free generation checks complement teacher-forced logits along the plain history.
            for (const std::string site : {"old", "full", "native"}) {
                execute(api, observed, &trace, output / prompt.first / (site + "-zero-greedy"), prompt.second,
                    {}, site, &basis, 0, site == "native", site == "native" ? 1 : 0);
            }
            execute(api, observed, &trace, output / prompt.first / "native-clear-greedy", prompt.second,
                {}, "native", nullptr, 0, true, 0);
            for (const auto & fresh : {std::pair<const char *, float>{"fresh-A", 1.0f}, {"fresh-B", -1.0f}}) {
                auto * context = api.llama_init_from_model(api.model, cp);
                if (!context) throw std::runtime_error("Fresh context failed");
                execute(api, context, &trace, output / prompt.first / fresh.first, prompt.second, trajectory,
                    "native", &full_direction, fresh.second, true, 1);
                api.llama_free(context);
            }
            execute(api, observed, &trace, output / prompt.first / "diagnostic-KV-seed-A", prompt.second,
                {}, "native", &full_direction, 1, true, 1, 1);
            execute(api, observed, &trace, output / prompt.first / "diagnostic-KV-retained-B", prompt.second,
                {}, "native", &full_direction, -1, true, 1, 1, false);
        }
        api.llama_free(observed);
        std::ofstream(output / "execution-completed.txt") << "Execution completed; inspect raw data before conclusions.\n";
        return 0;
    } catch (const std::exception & error) {
        std::cerr << "injection_probe: " << error.what() << '\n';
        return 1;
    }
}
