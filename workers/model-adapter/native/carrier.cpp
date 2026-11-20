// b11669 C ABI shim. Requests own their context, sampler and observed activations.
#include "llama.h"
#include "ggml-backend.h"
#include "mtmd.h"
#include "json-schema-to-grammar.h"
#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstring>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <string>
#include <vector>

#define API extern "C" __declspec(dllexport)
using Clock = std::chrono::steady_clock;
static thread_local std::string error;
static void require(bool ok, const char *reason) {
    if (!ok) throw std::runtime_error(reason);
}
static void quiet_log(ggml_log_level, const char *, void *) {}
struct Model {
    std::unique_ptr<llama_model, decltype(&llama_model_free)> model{nullptr, llama_model_free};
    std::unique_ptr<mtmd_context, decltype(&mtmd_free)> vision{nullptr, mtmd_free};
};
struct Stats {
    uint64_t positions = 0, batches = 0;
    double max_abs_error = 0, max_delta_l2 = 0;
};
struct Request {
    std::unique_ptr<llama_context, decltype(&llama_free)> ctx{nullptr, llama_free};
    std::unique_ptr<llama_sampler, decltype(&llama_sampler_free)> sampler{nullptr, llama_sampler_free};
    const llama_vocab *vocab = nullptr;
    std::string before_name, after_name, fault, piece;
    std::vector<float> delta, before;
    std::vector<double> requested;
    bool nonzero = false;
    int width = 0, seen = 0;
    llama_token pending = -1;
    Clock::time_point deadline;
    Stats stats;

    void observe(ggml_tensor *t) {
        require(t->type == GGML_TYPE_F32 && ggml_is_contiguous(t) && t->ne[0] == width
                && t->ne[1] > 0 && t->ne[2] == 1 && t->ne[3] == 1, "activation shape/dtype");
        std::vector<float> values(static_cast<size_t>(ggml_nelements(t)));
        ggml_backend_tensor_get(t, values.data(), 0, values.size() * sizeof(float));
        for (auto x : values) require(std::isfinite(x), "nonfinite activation");
        if (before_name == t->name) {
            require(before.empty(), "AT13: duplicate pre-injection tensor");
            before.swap(values);
            return;
        }
        if (nonzero) {
            require(before.size() == values.size(), "AT13: missing pre-injection tensor");
            bool changed = false;
            for (size_t offset = 0; offset < values.size(); offset += width) {
                double square = 0;
                for (int j = 0; j < width; ++j) {
                    const size_t i = offset + j;
                    const double observed = static_cast<double>(values[i]) - before[i];
                    // One F32 addition at the registered block residual. Exact rounded
                    // result is stronger than applying a guessed model-specific tolerance.
                    require(values[i] == before[i] + delta[j], "AT13: residual addition mismatch");
                    changed |= observed != 0;
                    square += observed * observed;
                    stats.max_abs_error = std::max(stats.max_abs_error, std::abs(observed - requested[j]));
                }
                stats.max_delta_l2 = std::max(stats.max_delta_l2, std::sqrt(square));
            }
            require(changed, "AT13: zero observed difference");
            before.clear();
        }
        seen += static_cast<int>(t->ne[1]);
        stats.positions += t->ne[1];
    }
    static bool probe(ggml_tensor *t, bool ask, void *user) noexcept {
        auto &r = *static_cast<Request *>(user);
        if (ask) return r.after_name == t->name || (r.nonzero && r.before_name == t->name);
        try { r.observe(t); return true; }
        catch (const std::exception &e) { r.fault = e.what(); return false; }
    }
    void decode(llama_token *tokens, int count) {
        require(Clock::now() < deadline, "request deadline");
        seen = 0;
        int rc = llama_decode(ctx.get(), llama_batch_get_one(tokens, count));
        if (!fault.empty()) throw std::runtime_error(fault);
#ifdef P513_PLAIN_REFERENCE
        require(rc == 0, "baseline decode");
#else
        require(rc == 0 && seen == count && before.empty(), "AT13: incomplete forward probe");
#endif
        require(Clock::now() < deadline, "request deadline");
        ++stats.batches;
    }
};

API const char *carrier_error() { return error.c_str(); }
API int carrier_measurement_control() {
#if defined(P513_DROP_VECTOR) || defined(P513_PLAIN_REFERENCE)
    return 1;
#else
    return 0;
#endif
}
API Model *carrier_open(const char *engine, const char *model_file, const char *mmproj) noexcept {
    try {
        static std::once_flag init;
        std::call_once(init, [&] {
            llama_log_set(quiet_log, nullptr);
            ggml_backend_load_all_from_path(engine);
            llama_backend_init();
        });
        ggml_backend_dev_t cuda = nullptr;
        for (size_t i = 0; i < ggml_backend_dev_count(); ++i) {
            auto d = ggml_backend_dev_get(i);
            if (ggml_backend_dev_type(d) == GGML_BACKEND_DEVICE_TYPE_GPU &&
                std::string(ggml_backend_dev_name(d)).find("CUDA") != std::string::npos) cuda = d;
        }
        require(cuda != nullptr, "CUDA backend unavailable");
        ggml_backend_dev_t devices[] = {cuda, nullptr};
        auto m = std::make_unique<Model>();
        auto mp = llama_model_default_params();
        mp.n_gpu_layers = -1; mp.devices = devices;
        m->model.reset(llama_model_load_from_file(model_file, mp));
        require(m->model && llama_model_n_embd(m->model.get()) == 2560 &&
                llama_model_n_layer(m->model.get()) == 32, "P0-12 model shape");
        if (mmproj) {
            auto vp = mtmd_context_params_default();
            vp.use_gpu = true; vp.device = cuda; vp.n_threads = 4;
            m->vision.reset(mtmd_init_from_file(mmproj, m->model.get(), vp));
            require(m->vision != nullptr, "visual encoder unavailable");
        }
        return m.release();
    } catch (const std::exception &e) { error = e.what(); return nullptr; }
}
API void carrier_close(Model *m) { delete m; }

// A null vector selects b11669's no-adapter baseline. Every request starts fresh.
API Request *carrier_begin(Model *m, const char *prompt, int length, const int *spans,
                           int span_count, int layer,
                           const double *delta, int width, const char *json_schema,
                           int max_tokens, int64_t duration_ns) noexcept {
    try {
        require(m && prompt && length > 0 && layer > 0 && layer < 31 && width == 2560 &&
                max_tokens > 0 && max_tokens < 8192 && duration_ns > 0, "request bounds");
        auto r = std::make_unique<Request>();
        const auto now = Clock::now();
        const auto remaining = Clock::time_point::max() - now;
        r->deadline = now + std::min(Clock::duration(std::chrono::nanoseconds(duration_ns)), remaining);
        r->vocab = llama_model_get_vocab(m->model.get());
        r->width = width;
        r->before_name = "post_ffn-" + std::to_string(layer);
        r->after_name = "l_out-" + std::to_string(layer);
        if (delta) {
            r->requested.assign(delta, delta + width);
            for (double x : r->requested) {
                const float actual = static_cast<float>(x);
                require(std::isfinite(actual), "invalid F32 delta");
                r->nonzero |= actual != 0;
                r->delta.push_back(actual);
            }
            require(r->nonzero, "nonzero m rounded to zero");
        }
        auto cp = llama_context_default_params();
        // Same numeric configuration as P0-12, including hybrid state and microbatch.
        cp.n_ctx = 8192; cp.n_batch = 512; cp.n_ubatch = 512;
        cp.n_threads = 4; cp.n_threads_batch = 4; cp.flash_attn_type = LLAMA_FLASH_ATTN_TYPE_DISABLED;
        cp.cb_eval = Request::probe; cp.cb_eval_user_data = r.get();
#ifdef P513_PLAIN_REFERENCE
        cp.cb_eval = nullptr; cp.cb_eval_user_data = nullptr;
#endif
        r->ctx.reset(llama_init_from_model(m->model.get(), cp));
        require(r->ctx != nullptr, "context unavailable");
        auto memory = llama_get_memory(r->ctx.get());
        require(memory != nullptr, "hybrid memory unavailable");
        llama_memory_clear(memory, true);
        std::vector<float> buffer;
        if (r->nonzero) {
            buffer.resize(31 * width);
            std::copy(r->delta.begin(), r->delta.end(), buffer.begin() + (layer - 1) * width);
        }
#ifdef P513_DROP_VECTOR
        buffer.clear();  // Independently built AT13 negative control; leave the observer armed.
#endif
        require(llama_set_adapter_cvec(r->ctx.get(), buffer.empty() ? nullptr : buffer.data(),
                                       buffer.size(), width, layer, layer) == 0, "vector install");
        r->sampler.reset(llama_sampler_chain_init(llama_sampler_chain_default_params()));
        require(r->sampler != nullptr, "sampler unavailable");
        if (json_schema) {
            require(!r->nonzero, "judgment requires baseline");
            const auto grammar = json_schema_to_grammar(common_json::parse(json_schema), true);
            require(!grammar.empty(), "empty constrained grammar");
            auto sampler = llama_sampler_init_grammar(r->vocab, grammar.c_str(), "root");
            require(sampler != nullptr, "unsupported constrained grammar");
            llama_sampler_chain_add(r->sampler.get(), sampler);
        }
        llama_sampler_chain_add(r->sampler.get(), llama_sampler_init_greedy());
        std::vector<llama_token> tokens(cp.n_ctx);
        require(span_count >= 0 && span_count <= 4 && span_count % 2 == 0, "message spans");
        int n = 0, pos = 0;
        for (int s = 0; s <= span_count; ++s) {
            const int end = s == span_count ? length : spans[s];
            require(end >= pos && end <= length, "message span bounds");
            if (end > pos) {
                const int added = llama_tokenize(r->vocab, prompt + pos, end - pos, tokens.data() + n,
                                                 static_cast<int>(tokens.size()) - n, false, s % 2 == 0);
                require(added >= 0, "prompt exceeds context");
                n += added;
            }
            pos = end;
        }
        require(n > 0 && n <= static_cast<int>(cp.n_ctx) - max_tokens, "context budget");
        for (int i = 0; i < n; i += cp.n_batch)
            r->decode(tokens.data() + i, std::min(n - i, static_cast<int>(cp.n_batch)));
        return r.release();
    } catch (const std::exception &e) { error = e.what(); return nullptr; }
}
API int carrier_next(Request *r, const char **piece, size_t *bytes) noexcept {
    try {
        require(Clock::now() < r->deadline, "request deadline");
        if (r->pending >= 0) r->decode(&r->pending, 1);
        r->pending = llama_sampler_sample(r->sampler.get(), r->ctx.get(), -1);
        if (llama_vocab_is_eog(r->vocab, r->pending)) return 0;
        r->piece.resize(32);  // Grow to the actual tokenizer-reported byte length if needed.
        int n = llama_token_to_piece(r->vocab, r->pending, r->piece.data(), 32, 0, false);
        if (n < 0) {
            r->piece.resize(-n);
            n = llama_token_to_piece(r->vocab, r->pending, r->piece.data(), -n, 0, false);
        }
        require(n >= 0, "token decode");
        *piece = r->piece.data(); *bytes = static_cast<size_t>(n);
        return 1;
    } catch (const std::exception &e) { error = e.what(); return -1; }
}
API void carrier_stats(Request *r, Stats *s) { *s = r->stats; }
API const float *carrier_logits(Request *r, int *count) {
    *count = llama_vocab_n_tokens(r->vocab);
    return llama_get_logits_ith(r->ctx.get(), -1);
}
API void carrier_end(Request *r) { delete r; }
