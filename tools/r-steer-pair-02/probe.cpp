// Isolated H0/M0 research apparatus; pinned DLLs and production code are unchanged.
#include "native_probe.hpp"
#include <array>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <sstream>

struct H0 {
    Native & api;
    static constexpr std::array<int,5> layers{0,7,15,21,24};
    std::array<int,5> counts{};
    std::vector<float> vector;
    fs::path output;
    std::ofstream audit;
    int active=-1, expected_columns=0, step=0, writes=0;
    std::string error;
    explicit H0(Native & value):api(value) {}
    static bool callback(ggml_tensor * tensor,bool ask,void * user) noexcept {
        auto & self=*static_cast<H0 *>(user);
        int index=-1;
        for(int i=0;i<5;++i) if(std::string(tensor->name)=="l_out-"+std::to_string(layers[i])) index=i;
        if(ask) return index>=0;
        if(index<0) return true;
        try {
            if(++self.counts[index]!=1 || tensor->type!=GGML_TYPE_F32 || tensor->ne[0]!=2560 ||
               tensor->ne[1]!=self.expected_columns || tensor->ne[2]!=1 || tensor->ne[3]!=1 ||
               tensor->nb[0]!=4 || tensor->nb[1]!=10240)
                throw std::runtime_error("H0 layout/coverage differs");
            if(layers[index]!=self.active) return true;
            const size_t n=static_cast<size_t>(self.expected_columns)*2560;
            std::vector<float> before(n), after(n), readback(n);
            self.api.ggml_backend_tensor_get(tensor,before.data(),0,n*sizeof(float));
            double maximum_error=0;
            for(size_t j=0;j<n;++j) {
                const float v=self.vector[j%2560];
                const double exact=double(before[j])+double(v);
                after[j]=before[j]+v;
                const double error=std::abs(double(after[j])-exact);
                const double bound=std::ldexp(1.,-23)*(std::abs(double(before[j]))+std::abs(double(v)))+std::ldexp(1.,-149);
                if(!std::isfinite(before[j]) || !std::isfinite(after[j]) || error>bound)
                    throw std::runtime_error("H0 finite/addition bound failed");
                maximum_error=std::max(maximum_error,error);
            }
            self.api.ggml_backend_tensor_set(tensor,after.data(),0,n*sizeof(float));
            self.api.ggml_backend_tensor_get(tensor,readback.data(),0,n*sizeof(float));
            if(std::memcmp(after.data(),readback.data(),n*sizeof(float))!=0)
                throw std::runtime_error("H0 readback differs");
            if(self.step<2) {
                save_floats(self.output/("h0-before-"+std::to_string(self.step)+".f32"),
                    std::vector<float>(before.end()-2560,before.end()));
                save_floats(self.output/("h0-after-"+std::to_string(self.step)+".f32"),
                    std::vector<float>(readback.end()-2560,readback.end()));
            }
            self.audit<<self.step<<'\t'<<self.active<<'\t'<<self.expected_columns<<'\t'<<n<<'\t'<<maximum_error<<"\t1\n";
            self.audit.flush();
            if(!self.audit) throw std::runtime_error("H0 audit write failed");
            ++self.writes;
        } catch(const std::exception & error) {self.error=error.what();return false;}
        return true;
    }
};

bool safe(const std::string & value) {
    return !value.empty() && value.find_first_not_of("abcdefghijklmnopqrstuvwxyz0123456789-_")==std::string::npos;
}

std::string read(const fs::path & file) {
    std::ifstream input(file,std::ios::binary);
    if(!input) throw std::runtime_error("Missing frozen input");
    return {std::istreambuf_iterator<char>(input),std::istreambuf_iterator<char>()};
}

int main(int argc,char ** argv) {
    try {
        if(argc!=6 || std::string(argv[5])!="--quiet-window-confirmed") throw std::runtime_error("Invalid arguments");
        const fs::path inputs=fs::absolute(argv[3]), root=fs::absolute(argv[4]);
        Native api(fs::absolute(argv[1]),argv[2],32,248320,10);
        const auto piece=symbol<decltype(&llama_token_to_piece)>(GetModuleHandleW(L"llama.dll"),"llama_token_to_piece");
        H0 hook(api);
        auto parameters=api.parameters();parameters.cb_eval=H0::callback;parameters.cb_eval_user_data=&hook;
        auto * ctx=api.llama_init_from_model(api.model,parameters);
        auto * plain=api.llama_init_from_model(api.model,api.parameters());
        if(!ctx || !plain) throw std::runtime_error("Context unavailable");
        std::cout<<"READY"<<std::endl;
        std::string line;
        while(std::getline(std::cin,line) && line!="QUIT") {
            require_quiet();
            std::istringstream command(line);
            std::string id,material,mode,layer_string,vector,extra;
            if(!std::getline(command,id,'\t') || !std::getline(command,material,'\t') ||
               !std::getline(command,mode,'\t') || !std::getline(command,layer_string,'\t') ||
               !std::getline(command,vector,'\t') || std::getline(command,extra) ||
               !safe(id) || !safe(material) || !safe(vector) || (mode!="free" && mode!="guard"))
                throw std::runtime_error("Malformed command");
            const int layer=std::stoi(layer_string);
            if((layer==-1)!=(vector=="none") || (layer!=-1 && std::find(H0::layers.begin(),H0::layers.end(),layer)==H0::layers.end()))
                throw std::runtime_error("Unregistered layer/vector");
            const auto out=root/"requests"/id;
            if(!fs::create_directory(out)) throw std::runtime_error("Preserve old evidence");
            auto tokens=api.tokenize(read(inputs/"prompts"/(material+".txt")));
            const int limit=mode=="free"?384:1;
            if(tokens.size()+limit>512) throw std::runtime_error("Context budget exceeded");
            std::ofstream ids(out/"prompt-ids.tsv");for(auto token:tokens) ids<<token<<'\n';
            hook.vector.clear();hook.active=layer;hook.output=out;hook.writes=0;
            if(layer!=-1) {
                const auto bytes=read(inputs/"vectors"/(vector+".f32"));
                if(bytes.size()!=10240) throw std::runtime_error("Vector shape differs");
                hook.vector.resize(2560);std::memcpy(hook.vector.data(),bytes.data(),bytes.size());
                for(float value:hook.vector) if(!std::isfinite(value)) throw std::runtime_error("Nonfinite vector");
            }
            hook.audit.open(out/"h0-audit.tsv");hook.audit<<std::setprecision(17)<<"step\tlayer\tcolumns\telements\tmax_roundoff\treadback_exact\n";
            api.llama_memory_clear(api.llama_get_memory(ctx),true);
            if(api.control(ctx,nullptr,0)!=0) throw std::runtime_error("Adapter removal failed");
            auto batch=api.llama_batch_get_one(tokens.data(),static_cast<int32_t>(tokens.size()));
            std::vector<float> native_zero;
            if(mode=="guard" && id.find("-start")!=std::string::npos) {
                api.llama_memory_clear(api.llama_get_memory(plain),true);
                if(api.control(plain,nullptr,0)!=0 || api.llama_decode(plain,batch)!=0) throw std::runtime_error("Native zero baseline failed");
                const auto * raw=api.llama_get_logits_ith(plain,-1);
                if(!raw) throw std::runtime_error("Native baseline logits missing");
                native_zero.assign(raw,raw+api.vocabulary);save_floats(out/"native-zero-logits.f32",native_zero);
            }
            std::ofstream output(out/"output.bin",std::ios::binary), record(out/"tokens.tsv");
            record<<std::setprecision(17)<<"step\tid\tlogit\tlogZ\teog\n";
            llama_token next=0;bool ended=false;int steps=0;
            for(int step=0;step<limit;++step) {
                require_quiet();
                hook.step=step;hook.expected_columns=step==0?static_cast<int>(tokens.size()):1;
                hook.counts={};hook.error.clear();
                if(api.llama_decode(ctx,batch)!=0 || !hook.error.empty() || hook.counts!=std::array<int,5>{1,1,1,1,1})
                    throw std::runtime_error("H0 decode/coverage failed: "+hook.error);
                if(hook.writes!=(layer==-1?0:step+1)) throw std::runtime_error("H0 write coverage differs");
                const auto * raw=api.llama_get_logits_ith(ctx,-1);
                if(!raw) throw std::runtime_error("Missing logits");
                std::vector<float> values(raw,raw+api.vocabulary);
                for(float value:values) if(!std::isfinite(value)) throw std::runtime_error("Nonfinite logits");
                if(!native_zero.empty() && std::memcmp(values.data(),native_zero.data(),values.size()*sizeof(float))!=0)
                    throw std::runtime_error("Native versus callback zero drift");
                if(mode=="guard") save_floats(out/"first-logits.f32",values);
                next=static_cast<llama_token>(std::max_element(values.begin(),values.end())-values.begin());
                double mass=0;for(float value:values) mass+=std::exp(double(value)-values[next]);
                ended=api.llama_vocab_is_eog(api.vocab,next);
                record<<step<<'\t'<<next<<'\t'<<values[next]<<'\t'<<double(values[next])+std::log(mass)<<'\t'<<ended<<'\n';
                if(!ended) {
                    std::array<char,512> buffer{};
                    const auto n=piece(api.vocab,next,buffer.data(),static_cast<int32_t>(buffer.size()),0,true);
                    if(n<0) throw std::runtime_error("Token piece overflow");
                    output.write(buffer.data(),n);
                }
                ++steps;if(ended) break;
                batch=api.llama_batch_get_one(&next,1);
            }
            record.flush();output.flush();ids.flush();hook.audit.close();
            if(!record || !output || !ids || !hook.audit) throw std::runtime_error("Evidence write failed");
            std::ofstream done(out/"completion.tsv");
            done<<"steps\teog\tlimit\th0_writes\n"<<steps<<'\t'<<ended<<'\t'<<limit<<'\t'<<hook.writes<<'\n';
            done.flush();if(!done) throw std::runtime_error("Completion write failed");
            std::cout<<"OK\t"<<id<<'\t'<<steps<<std::endl;
        }
        api.llama_free(plain);api.llama_free(ctx);return 0;
    } catch(const std::exception & error) {std::cerr<<"c3_2a_probe: "<<error.what()<<'\n';return 1;}
}

