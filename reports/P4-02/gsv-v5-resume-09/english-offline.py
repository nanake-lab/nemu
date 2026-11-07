import network_guard
import os,sys,json,pathlib,importlib.util
os.environ['CUDA_VISIBLE_DEVICES']=''
os.environ['NLTK_DATA']=r'D:\Dev\cache\p4-02\nltk_data'
os.environ['HF_HUB_OFFLINE']='1'
root=pathlib.Path(r'D:\Dev\cache\p4-02\gpt-sovits-v5-official')
os.chdir(root); sys.path[:0]=[str(root),str(root/'GPT_SoVITS')]
import nltk
print(json.dumps({'resources':[str(nltk.data.find(p)) for p in ['taggers/averaged_perceptron_tagger.zip','taggers/averaged_perceptron_tagger_eng/','corpora/cmudict.zip']]}),flush=True)
assert importlib.util.find_spec('distance') is None
from text import english
print(json.dumps({'english_import':'ok','phones':english.g2p('hello world')}),flush=True)
assert not network_guard.NETWORK_LOG.exists()
