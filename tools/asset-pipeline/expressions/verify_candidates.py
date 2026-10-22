"""Independent byte/structure checks for assembled, unapproved preview models."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import struct

from prepare_candidates import CANDIDATES, ROOT, load_glb


def main() -> None:
    evidence = json.loads((ROOT / 'reports/P2-06/assembly-checks.json').read_text())
    source, original_bin = load_glb(Path(evidence['source']))
    output = []
    for path in sorted((CANDIDATES / 'models').glob('*.vrm')):
        model, binary = load_glb(path)
        assert binary[:len(original_bin)] == original_bin, 'Source binary rewritten'
        assert model['skins'] == source['skins'], 'Skin changed'
        assert model['extensions']['VRMC_springBone'] == source['extensions']['VRMC_springBone']
        assert model['extensions']['VRMC_vrm']['meta'] == source['extensions']['VRMC_vrm']['meta']
        meta = model['extensions']['VRMC_vrm']['meta']
        assert meta['licenseUrl'] == 'https://vrm.dev/licenses/1.0/'
        assert meta['otherLicenseUrl'] == 'LICENSE.md'
        for mesh in model['meshes']:
            target_count = len(mesh.get('extras', {}).get('targetNames', []))
            for primitive in mesh['primitives']:
                assert len(primitive.get('targets', [])) == target_count
                for target in primitive.get('targets', []):
                    assert model['accessors'][target['POSITION']]['count'] == model['accessors'][primitive['attributes']['POSITION']]['count']
        for item in model['accessors'][len(source['accessors']):]:
            view = model['bufferViews'][item['bufferView']]
            width = {'VEC3': 3, 'VEC4': 4}[item['type']]
            element_size = (4 if item['componentType'] == 5126 else 2) * width
            start = view.get('byteOffset', 0) + item.get('byteOffset', 0)
            assert start + item['count'] * element_size <= len(binary)
            if item['componentType'] == 5126:
                values = struct.unpack_from('<' + 'f' * item['count'] * width, binary, start)
                assert all(float('-inf') < value < float('inf') for value in values)
        output.append(dict(model=path.name, sourceBytesPreserved=True, skinsPreserved=True,
                           springsPreserved=True, metadataAndPermissionsPreserved=True,
                           targetCountsConsistent=True, newAccessorDataFinite=True,
                           sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
    (ROOT / 'reports/P2-06/structural-checks.json').write_text(json.dumps(output, indent=2)+'\n')
    print(f'{len(output)} candidate models: byte preservation, permissions, morph and finite-accessor checks passed')


if __name__ == '__main__':
    main()
