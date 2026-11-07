import os, sys, pathlib, importlib.util, json
os.environ['CUDA_VISIBLE_DEVICES']=''
root=pathlib.Path(r'D:\Dev\cache\p4-02\gpt-sovits-v5-official')
sys.path[:0]=[str(root),str(root/'GPT_SoVITS')]
os.chdir(root)
assert importlib.util.find_spec('distance') is None
print('Distance absent',flush=True)
import torch
from module import models
print(json.dumps({'models_import':'ok','frozendict_loaded':'frozendict' in sys.modules,'cuda_initialized':torch.cuda.is_initialized()}),flush=True)
