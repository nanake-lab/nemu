// Same GGML callback boundaries for both paired intervention sites.
#pragma once
#include "native_probe.hpp"
#include <array>
#include <cstring>
#include <iostream>

struct Trace {
    Native & api;
    fs::path output;
    std::ofstream records;
    std::string site;
    const std::vector<float> * direction = nullptr;
    float amplitude = 0;
    bool all_positions = false;
    int step = 0;
    std::array<int, 2> counts{};
    std::array<std::vector<float>, 2> last;
    std::string error;

    explicit Trace(Native & engine) : api(engine) {}
    void begin(const fs::path & path, const std::string & inject_site,
        const std::vector<float> * vector, float scale, bool all) {
        output = path; site = inject_site; direction = vector; amplitude = scale; all_positions = all;
        step = 0; counts = {}; last = {}; error.clear();
        records.close(); records.open(output / "activations.tsv");
        records << "step\tsite\trows\tcolumns\tinjected_rows\tamplitude\tmax_add_error\tactual_delta_l2\n";
    }
    static bool callback(ggml_tensor * tensor, bool ask, void * user) noexcept {
        auto & self = *static_cast<Trace *>(user);
        const int index = !std::strcmp(tensor->name, "ffn_inp-17") ? 0 :
            !std::strcmp(tensor->name, "l_out-17") ? 1 : -1;
        if (ask) return index >= 0;
        if (index < 0) return true;
        try { self.observe(tensor, index); }
        catch (const std::exception & e) { self.error = e.what(); return false; }
        return true;
    }
    void observe(ggml_tensor * tensor, int index) {
        if (tensor->type != GGML_TYPE_F32 || tensor->ne[0] != api.embd || tensor->ne[1] < 1 ||
            tensor->ne[1] > 512 || tensor->ne[2] != 1 || tensor->ne[3] != 1 ||
            tensor->nb[0] != sizeof(float) || tensor->nb[1] != sizeof(float) * api.embd) {
            throw std::runtime_error("Callback activation shape/type is not registered contiguous F32");
        }
        if (++counts[index] != 1) throw std::runtime_error("Repeated callback site in one decode");
        const size_t elements = static_cast<size_t>(tensor->ne[0] * tensor->ne[1]);
        std::vector<float> before(elements);
        api.ggml_backend_tensor_get(tensor, before.data(), 0, elements * sizeof(float));
        for (float value : before) if (!std::isfinite(value)) throw std::runtime_error("Nonfinite activation");
        std::vector<float> after = before;
        const std::string name = index == 0 ? "old" : "full";
        const bool inject = direction && site == name;
        const size_t first_row = all_positions ? 0 : static_cast<size_t>(tensor->ne[1] - 1);
        double square = 0, add_error = 0;
        if (inject) {
            for (size_t row = first_row; row < static_cast<size_t>(tensor->ne[1]); ++row) {
                for (int i = 0; i < api.embd; ++i) {
                    const size_t at = row * api.embd + i;
                    const float delta = (*direction)[i] * amplitude;
                    after[at] += delta;
                    const double observed = static_cast<double>(after[at]) - before[at];
                    square += observed * observed;
                    add_error = std::max(add_error, std::abs(observed - delta));
                }
            }
            // A zero arm executes the same write path; byte consistency is measured.
            api.ggml_backend_tensor_set(tensor, after.data(), 0, elements * sizeof(float));
        }
        const auto offset = static_cast<ptrdiff_t>(elements - api.embd);
        last[index].assign(before.begin() + offset, before.end());
        if (step == 0) {
            save_floats(output / (name + "-prefill-all-before.f32"), before);
            save_floats(output / (name + "-prefill-all-after.f32"), after);
        }
        save_floats(output / (std::to_string(step) + "-" + name + "-before.f32"), last[index]);
        save_floats(output / (std::to_string(step) + "-" + name + "-after.f32"),
            std::vector<float>(after.begin() + offset, after.end()));
        records << step << '\t' << name << '\t' << tensor->ne[1] << '\t' << api.embd << '\t'
            << (inject ? static_cast<size_t>(tensor->ne[1]) - first_row : 0) << '\t' << amplitude << '\t'
            << add_error << '\t' << std::sqrt(square) << '\n';
        records.flush();
    }
    void verify() const {
        if (!error.empty()) throw std::runtime_error(error);
        if (counts != std::array<int, 2>{1, 1}) throw std::runtime_error("Required callback sites absent");
    }
};
