import os, sys, pathlib, json, time, hashlib, importlib.util, traceback
os.environ['CUDA_VISIBLE_DEVICES']=''
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
except BaseException as e:
 record('failure',error=repr(e)); raise
