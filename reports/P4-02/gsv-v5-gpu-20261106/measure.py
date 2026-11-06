"""Standalone V5 GPU observation; no sidecar transport or audio-device latency."""
import network_guard
import os, sys, pathlib, json, time, hashlib, traceback, math
os.environ['CUDA_VISIBLE_DEVICES']='0'
os.environ['NLTK_DATA']=r'D:\Dev\cache\p4-02\nltk_data'
os.environ['HF_HUB_OFFLINE']='1'
os.environ['TRANSFORMERS_OFFLINE']='1'
os.environ['MPLBACKEND']='Agg'
root=pathlib.Path(r'D:\Dev\cache\p4-02\gpt-sovits-v5-official')
out=pathlib.Path(__file__).parent
os.chdir(root); sys.path[:0]=[str(root),str(root/'GPT_SoVITS')]
handle=os.add_dll_directory(str(next(pathlib.Path(r'D:\Dev\cache\p4-02\gsv-v5-resume-04\ffmpeg').glob('*/bin'))))
import onnxruntime as ort
ort.get_available_providers=lambda: ['CPUExecutionProvider']
import torch, numpy as np
from scipy.signal import resample_poly
torch.set_num_threads(4)
def record(event, **data):
 data.update(event=event,utc=time.time(),perf=time.perf_counter(),pid=os.getpid())
 with (out/'events.jsonl').open('a',encoding='utf-8') as f:f.write(json.dumps(data,ensure_ascii=False)+'\n')
 print(json.dumps(data,ensure_ascii=False),flush=True)
def guard():
 if (out/'stop').exists(): raise RuntimeError('External monitor stopped the measurement')
 if network_guard.NETWORK_LOG.exists(): raise RuntimeError('Unexpected network request')
try:
 record('imports_ready',torch=torch.__version__,ort=ort.__version__)
 while not (out/'monitor-ready').exists():
  guard(); time.sleep(.1)
 from TTS_infer_pack.TTS import TTS,TTS_Config
 from text import chinese2
 assert chinese2.g2pw._g2pw.session_g2pw.get_providers()==['CPUExecutionProvider']
 weights=root/'GPT_SoVITS'/'pretrained_models'
 config=TTS_Config({'custom':{'version':'v5turbo','device':'cuda','is_half':True,'t2s_weights_path':str(weights/'s1v3.ckpt'),'vits_weights_path':str(weights/'gsv-v5-pretrained'/'s2Gv5turbo.pth'),'bert_base_path':str(weights/'chinese-roberta-wwm-ext-large'),'cnhuhbert_base_path':str(weights/'chinese-hubert-base')}})
 config.configs_path=str(out/'runtime-config.yaml')
 start=time.perf_counter(); model=TTS(config); torch.cuda.synchronize()
 record('model_loaded',elapsed_seconds=time.perf_counter()-start,allocated=torch.cuda.memory_allocated(),reserved=torch.cuda.memory_reserved(),max_allocated=torch.cuda.max_memory_allocated(),max_reserved=torch.cuda.max_memory_reserved(),g2pw=chinese2.g2pw._g2pw.session_g2pw.get_providers())
 # Ten seconds makes post-load residency observable to the independent sampler.
 time.sleep(10); guard(); record('load_observation_end')
 voice=pathlib.Path(r'D:\Dev\cache\p4-02\gsv-v5-test-voice')
 ref=json.loads((voice/'voice.json').read_text(encoding='utf-8-sig'))['references']['zh']['neutral']
 cases=json.loads((out/'cases.json').read_text(encoding='utf-8'))
 def synth(case,phase,index):
  guard()
  inputs={'text':case['text'],'text_lang':case['language'],'ref_audio_path':str(voice/'zh.wav'),'prompt_text':ref['text'],'prompt_lang':'all_zh','speed_factor':1.0,'seed':42,'parallel_infer':False,'streaming_mode':False,'return_fragment':False,'text_split_method':'cut0'}
  # Request dispatch begins before the generator is constructed/executed.
  start=time.perf_counter(); first=None; duration=0; count=0; digest=hashlib.sha256()
  for sr,audio in model.run(inputs):
   # Upstream can yield silent fallback before reloading after an exception.
   # Reject that first yield immediately so no recovery/retry is driven here.
   assert sr==48000 and len(audio)>0 and np.any(audio), 'Invalid/silent upstream fallback'
   pcm=resample_poly(audio.astype(np.float32)/32768.0,1,2).astype('<f4').tobytes()
   if first is None:first=time.perf_counter()-start
   digest.update(pcm); duration+=len(pcm)/4/24000; count+=1
  elapsed=time.perf_counter()-start
  assert first is not None and duration>0
  guard()
  record('request',phase=phase,index=index,case=case['case'],first_playable_seconds=first,total_seconds=elapsed,audio_seconds=duration,rtf=elapsed/duration,pcm_sha256=digest.hexdigest(),chunks=count,allocated=torch.cuda.memory_allocated(),reserved=torch.cuda.memory_reserved(),max_allocated=torch.cuda.max_memory_allocated(),max_reserved=torch.cuda.max_memory_reserved())
 synth(cases[0],'cold',0)
 for i,case in enumerate(cases):synth(case,'latency',i)
 record('stability_start'); begin=time.perf_counter(); i=0
 while time.perf_counter()-begin<1800:
  synth(cases[i%len(cases)],'stability',i); i+=1
 record('stability_end',duration_seconds=time.perf_counter()-begin,requests=i)
 record('complete',max_allocated=torch.cuda.max_memory_allocated(),max_reserved=torch.cuda.max_memory_reserved())
except BaseException as e:
 record('failure',error=repr(e)); traceback.print_exc(); raise
