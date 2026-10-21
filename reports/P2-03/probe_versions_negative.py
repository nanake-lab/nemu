import sys
from pathlib import Path
sys.path.insert(0,str(Path('tools/asset-pipeline').resolve()))
import blender_common as common
from errors import BuildError
base=Path('tools/asset-pipeline/.cache/negative-version').resolve();base.mkdir(parents=True,exist_ok=True)
common.__file__=str(base/'blender_common.py')
for field,blender,addon in [('Blender','0.0.0','4.10.2'),('VRM Add-on','5.2.2','0.0.0')]:
 (base/'blender-version.txt').write_text(blender)
 (base/'vrm-addon-version.txt').write_text(addon)
 try:
  common.check_versions()
 except BuildError as error:
  print('EXPECTED_REJECTION',field,str(error))
 else:
  raise AssertionError('Version mismatch was accepted: '+field)
print('VERSION_NEGATIVE_TESTS=2_PASS')
