"""Offline adapter of existing StatForge Summary JSON; no statistical computation."""
import argparse
from datetime import datetime
import hashlib
import json
import math
from pathlib import Path
import re
import sys
from urllib.parse import urlsplit

MAX_BYTES = 8 * 1024 * 1024
STATISTICS = ('mean', 'std', 'min', 'max', 'q25', 'q50', 'q75')


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False, allow_nan=False).encode()


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def file_hash(path, limit=64 * 1024 * 1024):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        total = 0
        while chunk := f.read(65536):
            total += len(chunk)
            if total > limit: raise ValueError('input exceeds byte limit')
            h.update(chunk)
    return h.hexdigest()


def read_json(path, limit=MAX_BYTES):
    with Path(path).open('rb') as f: data = f.read(limit + 1)
    if len(data) > limit: raise ValueError('JSON exceeds byte limit')
    def reject(value): raise ValueError('nonfinite JSON constant')
    return json.loads(data, parse_constant=reject)


def required(obj, key):
    value = obj.get(key)
    if not isinstance(value, str) or not value.strip(): raise ValueError('missing or invalid ' + key)
    return value


def timestamp(value):
    parsed = datetime.fromisoformat(value.replace('Z', '+00:00'))
    if parsed.tzinfo is None: raise ValueError('timestamp requires timezone')
    return value


def export_summary(run, manifest, input_path, recipe_path, artifact_key):
    if manifest.get('manifestVersion') != '1': raise ValueError('unsupported manifest version')
    if not re.fullmatch(r'[0-9a-f]{40}', manifest.get('engineCommit', '')): raise ValueError('engine commit is required')
    if manifest.get('method') != 'DESCRIPTIVE_SUMMARY': raise ValueError('unsupported method')
    if manifest.get('missingnessPolicy') != 'available-case': raise ValueError('unsupported missingness policy')
    dataset, spec = manifest['dataset'], manifest['spec']
    for item, path in [(dataset, input_path), (spec, recipe_path)]:
        if item.get('sha256') != file_hash(path): raise ValueError('input/spec hash mismatch')
    for key in ('id', 'version'): required(dataset, key)
    required(spec, 'reference')
    period, population = manifest['period'], manifest['population']
    if required(period, 'start') > required(period, 'end'): raise ValueError('period is reversed')
    if period.get('basis') != 'calendar-day': raise ValueError('Summary export requires explicit calendar-day basis')
    for key in ('start', 'end'): datetime.strptime(period[key], '%Y-%m-%d')
    required(population, 'description'); required(population, 'unit')
    geography, question = required(manifest, 'geography'), required(manifest, 'question')
    source = required(manifest, 'source')
    url = urlsplit(source)
    if url.scheme != 'https' or not url.hostname or url.username or url.password or url.query or url.fragment:
        raise ValueError('source must be a public HTTPS attribution URL without credentials or query')
    retrieved = timestamp(required(manifest, 'retrievedAt'))
    variables = manifest['variables']
    if not isinstance(variables, list) or not variables or len(variables) > 64: raise ValueError('invalid variable dictionary')
    names, scopes = set(), set()
    for v in variables:
        name = required(v, 'name')
        if name in names or not re.fullmatch(r'[A-Za-z][A-Za-z0-9_]{0,63}', name): raise ValueError('invalid/duplicate variable')
        names.add(name); scopes.add(required(v, 'scope'))
        required(v, 'unit'); required(v, 'definition')
    if len(scopes) != 1: raise ValueError('incomparable populations/time/geography')
    assumptions = manifest['assumptions']
    if not isinstance(assumptions, list) or not assumptions: raise ValueError('explicit assumptions required')
    flags = [{'code': 'DESCRIPTIVE_ONLY', 'message': 'Finite selected-group summary; no causal or national inference.'}]
    for a in assumptions:
        required(a, 'name'); required(a, 'basis')
        if a.get('status') == 'VIOLATED': raise ValueError('failed assumption')
        if a.get('status') not in ('PLAUSIBLE', 'UNKNOWN', 'NOT_CHECKED', 'NOT_APPLICABLE'): raise ValueError('invalid assumption status')
        if a['status'] in ('UNKNOWN', 'NOT_CHECKED'): flags.append({'code': 'ASSUMPTION_UNKNOWN', 'message': a['name']})
    uncertainty = manifest['uncertainty']
    if uncertainty.get('status') != 'NOT_ESTIMATED': raise ValueError('Summary does not estimate inferential uncertainty')
    required(uncertainty, 'reason')
    execution = run['execution']
    tool, version = required(execution, 'tool'), required(execution, 'version')
    if tool.lower() not in ('statforge', 'stats workbench'): raise ValueError('unsupported statistical engine')
    legacy = tool.lower() == 'statforge'
    generated = timestamp(required(execution, 'started_at'))
    output = run['artifacts'][artifact_key]
    if output.get('kind') != 'Summary': raise ValueError('unsupported artifact kind')
    columns = output['values']['columns']
    if not isinstance(columns, list) or len(columns) > 64: raise ValueError('invalid summary columns')
    by_name = {c['name']: c for c in columns}
    if len(by_name) != len(columns): raise ValueError('duplicate summary column')
    metrics, results, diagnostics, selected = [], [], [], []
    for v in sorted(variables, key=lambda x: x['name']):
        name = v['name']
        if name not in by_name: raise ValueError('declared variable absent')
        c = by_name[name]
        count, nulls = c['count'], c['null_count']
        if type(count) is not int or type(nulls) is not int or not 0 <= nulls <= count: raise ValueError('invalid counters')
        if count > nulls and all(c.get(stat) is None for stat in STATISTICS): raise ValueError('non-numeric or unsupported summary column')
        for stat in STATISTICS:
            if stat not in c: raise ValueError('required statistic absent: ' + stat)
            value = c[stat]
            if value is not None and (type(value) not in (int, float) or not math.isfinite(value)): raise ValueError('invalid statistic')
            if count == nulls and value is not None: raise ValueError('all-missing column has value')
            mid = name + '.' + stat
            metrics.append({'id': mid, 'name': mid, 'version': '1', 'unit': v['unit'], 'aggregation': stat,
                            'description': v['definition'] + '; descriptive across declared group'})
            result = {'metricId': mid, 'period': period, 'dimensions': {'geography': geography, 'scope': v['scope']},
                      'temporal': {'observedAt': generated, 'origin': 'derived', 'geography': geography, 'valueBasis': 'not_applicable'}}
            if value is None: result['missing'] = True
            else: result['value'] = value
            results.append(result)
        if nulls: flags.append({'code': 'MISSING_VALUES', 'message': name})
        diagnostics.append({'variable': name, 'rowCount': count, 'missingCount': nulls, 'availableCount': count-nulls})
        selected.append({k: c[k] for k in ('name', 'count', 'null_count') + STATISTICS})
    plan = {'question': question, 'method': 'DESCRIPTIVE_SUMMARY', 'variables': variables,
            'period': period, 'population': population, 'geography': geography,
            'missingnessPolicy': 'available-case', 'assumptions': assumptions, 'uncertainty': uncertainty,
            'interpretation': 'DESCRIPTIVE_ONLY', 'quantileDefinition': 'rounded order statistic: round((n-1)*q), ties away from zero'}
    result_hash = digest({'engine': tool, 'engineVersion': version, 'columns': selected})
    out = {'artifactSchema': 'insight-lab.analytical-artifact', 'schemaVersion': '1',
           'id': '', 'producer': 'StatForge-summary-export' if legacy else 'StatsWorkbench-summary-export', 'producerVersion': '1', 'generatedAt': generated,
           'datasets': [{'id': dataset['id'], 'version': dataset['version'], 'hash': {'algorithm': 'sha256', 'value': dataset['sha256']}}],
           'spec': {'kind': 'STATFORGE_RECIPE' if legacy else 'STATS_WORKBENCH_RECIPE', 'reference': spec['reference'], 'hash': {'algorithm': 'sha256', 'value': spec['sha256']}},
           'parameters': {'statisticalPlan': plan, 'planFingerprint': digest(plan),
                          'resultFingerprint': result_hash, 'diagnostics': diagnostics,
                          'engineCommit': manifest['engineCommit'], 'randomSeed': None},
           'period': period, 'population': population, 'metrics': metrics, 'results': results,
           'qualityFlags': flags, 'computation': {'engine': tool, 'engineVersion': version, 'deterministic': True},
           'provenance': [{'datasetId': dataset['id'], 'source': source, 'retrievedAt': retrieved,
                           'license': manifest.get('license', ''), 'transformationRefs': [spec['reference']]}]}
    # ID and hash are sealed without a recursive dependency between themselves.
    out['id'] = ('statforge-' if legacy else 'stats-workbench-') + digest(out)
    out['artifactHash'] = {'algorithm': 'sha256', 'value': digest(out)}
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ('result', 'manifest', 'input', 'recipe', 'out'): parser.add_argument('--' + key, required=True, type=Path)
    parser.add_argument('--artifact', default='summary')
    args = parser.parse_args()
    try:
        out = export_summary(read_json(args.result), read_json(args.manifest), args.input, args.recipe, args.artifact)
        args.out.write_bytes(canonical(out) + b'\n')
    except (ValueError, KeyError, TypeError, OSError) as error:
        print(json.dumps({'code': 'INVALID_ANALYTICAL_EXPORT', 'message': str(error)}), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__': sys.exit(main())
