import os,sys,pathlib,json
os.environ['CUDA_VISIBLE_DEVICES']=''
root=pathlib.Path(r'D:\Dev\cache\p4-02\gpt-sovits-v5-official')
os.chdir(root); sys.path[:0]=[str(root),str(root/'GPT_SoVITS')]
handle=os.add_dll_directory(str(next((pathlib.Path(__file__).parent/'ffmpeg').glob('*/bin'))))
import torch
from TTS_infer_pack.TTS import TTS, TTS_Config
print(json.dumps({'tts_import':'ok','cuda_initialized':torch.cuda.is_initialized()}))
