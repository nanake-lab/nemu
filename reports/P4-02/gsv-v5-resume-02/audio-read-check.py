import json, torch, torchaudio
from importlib.metadata import version
print(json.dumps({'torch':torch.__version__,'torchaudio':torchaudio.__version__,'torchcodec':version('torchcodec'),'cuda_initialized':torch.cuda.is_initialized()}), flush=True)
audio, rate = torchaudio.load(r'D:\Dev\cache\p4-02\gsv-v5-test-voice\zh.wav')
print(json.dumps({'sample_rate':rate,'shape':list(audio.shape),'cuda_initialized':torch.cuda.is_initialized()}), flush=True)
