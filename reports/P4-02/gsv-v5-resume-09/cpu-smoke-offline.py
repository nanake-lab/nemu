import network_guard
os_nltk_path = r'D:\Dev\cache\p4-02\nltk_data'
import os, sys, pathlib, json, time, hashlib, importlib.util, traceback
os.environ['CUDA_VISIBLE_DEVICES']=''
os.environ['NLTK_DATA']=os_nltk_path
os.environ['HF_HUB_OFFLINE']='1'
os.environ['TRANSFORMERS_OFFLINE']='1'
os.environ['MPLBACKEND']='Agg'
root=pathlib.Path(r'D:\Dev\cache\p4-02\gpt-sovits-v5-official')
out=pathlib.Path(__file__).parent
os.chdir(root); sys.path[:0]=[str(root),str(root/'GPT_SoVITS')]
handle=os.add_dll_directory(str(next(pathlib.Path(r'D:\Dev\cache\p4-02\gsv-v5-resume-04\ffmpeg').glob('*/bin'))))
import onnxruntime as ort
import torch
original_providers=ort.get_available_providers()
ort.get_available_providers=lambda: ['CPUExecutionProvider']
torch.set_num_threads(4)
def record(event, **data):
 data.update(event=event,cuda_initialized=torch.cuda.is_initialized())
 with (out/'cpu-smoke.jsonl').open('a',encoding='utf-8') as f:f.write(json.dumps(data,ensure_ascii=False)+'\n')
 print(json.dumps(data,ensure_ascii=False),flush=True)
 assert not torch.cuda.is_initialized()
try:
 assert importlib.util.find_spec('distance') is None
 record('isolation',ort_version=ort.__version__,original_providers=original_providers,preload_dlls_present=hasattr(ort,'preload_dlls'))
 from TTS_infer_pack.TTS import TTS,TTS_Config
 from text import chinese2
 providers=chinese2.g2pw._g2pw.session_g2pw.get_providers()
 assert providers==['CPUExecutionProvider'],providers
 record('g2pw_initialized',providers=providers)
 weights=root/'GPT_SoVITS'/'pretrained_models'
 config=TTS_Config({'custom':{'version':'v5turbo','device':'cpu','is_half':False,'t2s_weights_path':str(weights/'s1v3.ckpt'),'vits_weights_path':str(weights/'gsv-v5-pretrained'/'s2Gv5turbo.pth'),'bert_base_path':str(weights/'chinese-roberta-wwm-ext-large'),'cnhuhbert_base_path':str(weights/'chinese-hubert-base')}})
 config.configs_path=str(out/'runtime-config.yaml')
 start=time.perf_counter(); model=TTS(config); record('model_loaded',elapsed_seconds=time.perf_counter()-start)
 voice=pathlib.Path(r'D:\Dev\cache\p4-02\gsv-v5-test-voice')
 ref=json.loads((voice/'voice.json').read_text(encoding='utf-8-sig'))['references']['zh']['neutral']
 import soundfile as sf
 for name,text,language in [('zh','你好，今天的天空很明亮。','all_zh'),('mixed','你好，hello world。','zh')]:
  start=time.perf_counter(); chunks=list(model.run({'text':text,'text_lang':language,'ref_audio_path':str(voice/'zh.wav'),'prompt_text':ref['text'],'prompt_lang':'all_zh','speed_factor':1.0,'seed':42,'parallel_infer':False,'streaming_mode':False,'return_fragment':False,'text_split_method':'cut0'})); elapsed=time.perf_counter()-start
  assert chunks
  for i,(sr,audio) in enumerate(chunks):
   assert len(audio)>0
   path=out/f'{name}-{i}.wav'; sf.write(str(path),audio,sr)
   record('synthesis',case=name,chunk=i,sample_rate=sr,samples=len(audio),audio_seconds=len(audio)/sr,elapsed_seconds=elapsed,sha256=hashlib.sha256(path.read_bytes()).hexdigest(),providers=chinese2.g2pw._g2pw.session_g2pw.get_providers())
 assert not network_guard.NETWORK_LOG.exists(), 'Unexpected network attempts recorded'
 record('complete')
except BaseException as e:
 record('failure',error=repr(e)); raise
