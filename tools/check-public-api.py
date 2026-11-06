#!/usr/bin/env python3
"""Run inside a private test container; inputs are read-only Rustdoc JSON files."""
import json
from pathlib import Path


def ledger(file):
    doc = json.loads(Path(file).read_text())
    index = doc['index']
    paths = {str(k): '::'.join(v['path']) for k, v in doc['paths'].items()}
    entries = {}

    def norm(value):
        if isinstance(value, list):
            return [norm(v) for v in value]
        if isinstance(value, dict):
            return {k: (paths.get(str(v), str(v)) if k == 'id' else norm(v)) for k, v in value.items() if k not in ('span', 'docs', 'links', 'attrs')}
        return value

    def visit(identity, path):
        item = index[str(identity)]
        name = item['name']
        path = path + ([name] if name else [])
        key = '::'.join(path)
        kind, data = next(iter(item['inner'].items()))
        paths[str(identity)] = key
        if kind == 'module':
            entries[key] = {'kind': kind}
            for child in data['items']:
                visit(child, path)
        elif kind in ('struct', 'enum', 'union'):
            shape = {k: v for k, v in data.items() if k != 'impls'}
            if kind == 'struct' and isinstance(shape['kind'], dict):
                form, fields = next(iter(shape['kind'].items()))
                if form == 'plain':
                    fields = dict(fields)
                    fields['fields'] = [index[str(f)]['inner'] | {'name': index[str(f)]['name']} for f in fields['fields']]
                elif form == 'tuple':
                    fields = [index[str(f)]['inner'] if f is not None else None for f in fields]
                shape['kind'] = {form: fields}
            if kind == 'enum':
                shape['variants'] = [index[str(v)]['name'] for v in data['variants']]
                for child in data['variants']:
                    visit(child, path)
            entries[key] = {'kind': kind, 'shape': norm(shape)}
            for impl_id in data.get('impls', []):
                impl = index[str(impl_id)]['inner']['impl']
                if impl['trait'] is None:
                    for method in impl['items']:
                        child = index[str(method)]
                        if child['visibility'] == 'public':
                            visit(method, path)
        elif kind == 'variant':
            shape = data.copy()
            form = shape['kind']
            if isinstance(form, dict) and 'tuple' in form:
                shape['kind'] = {'tuple': [index[str(f)]['inner'] if f is not None else None for f in form['tuple']]}
            elif isinstance(form, dict) and 'struct' in form:
                fields = dict(form['struct'])
                fields['fields'] = [index[str(f)]['inner'] | {'name': index[str(f)]['name']} for f in fields['fields']]
                shape['kind'] = {'struct': fields}
            entries[key] = {'kind': kind, 'shape': norm(shape)}
        elif kind == 'use':
            entries[key] = {'kind': kind, 'source': data['source'], 'name': data['name'], 'is_glob': data['is_glob']}
        else:
            entries[key] = {'kind': kind, 'shape': norm(data)}

    visit(doc['root'], [])
    return entries


import argparse
parser = argparse.ArgumentParser(description='Compare structural exported APIs from two Rustdoc JSON artifacts.')
parser.add_argument('baseline')
parser.add_argument('candidate')
parser.add_argument('output')
args = parser.parse_args()
base = ledger(args.baseline)
candidate = ledger(args.candidate)
missing = sorted(set(base) - set(candidate))
changed = {k: {'baseline': v, 'candidate': candidate[k]} for k, v in base.items() if k in candidate and v != candidate[k]}
Path(args.output).write_text(json.dumps({'baseline': len(base), 'candidate': len(candidate), 'missing': missing, 'changed': changed}, indent=2))
print('API entries', len(base), len(candidate), 'missing', len(missing), 'changed', len(changed))
assert not missing and not changed
