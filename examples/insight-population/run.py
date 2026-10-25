"""Reproduce a reviewed public-data summary using the existing sf binary."""
import argparse
import importlib.util
import json
import os
import re
from pathlib import Path
import sqlite3
import subprocess
import time
import urllib.request
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('exporter', ROOT / 'scripts/export_analytical.py')
exporter = importlib.util.module_from_spec(spec); spec.loader.exec_module(exporter)
SOURCE_HASH = '163f1816517f73f6d1641b43e79ac1663371ef1ab6c0227d4b5bcff1ff7e55a5'
SOURCE = Path(__file__).with_name('source.json')
RECIPE = Path(__file__).with_name('summary.toml')


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ValueError('engine redirects are not accepted')


def request(base, method, path, payload=None):
    req = urllib.request.Request(base + '/api/public/v1' + path,
          data=exporter.canonical(payload) if payload is not None else None,
          headers={'Content-Type': 'application/json'}, method=method)
    with urllib.request.build_opener(NoRedirect).open(req, timeout=10) as r:
        data = r.read(exporter.MAX_BYTES + 1)
    if len(data) > exporter.MAX_BYTES: raise ValueError('engine response exceeds limit')
    return json.loads(data)


def prepare(out):
    if exporter.file_hash(SOURCE) != SOURCE_HASH: raise ValueError('official source hash mismatch')
    source = exporter.read_json(SOURCE)['GET_STATS']['STATISTICAL_DATA']['DATA_INF']['DATA_OBJ']
    db = out / 'population.sqlite'
    # Separate caller-selected output only. Refuse overwrite of a preexisting database.
    if db.exists(): raise ValueError('output database already exists; use a fresh directory')
    with sqlite3.connect(db) as conn:
        conn.execute('CREATE TABLE population(region_code TEXT NOT NULL, year INTEGER NOT NULL, population INTEGER NOT NULL)')
        rows = []
        for item in source:
            v = item['VALUE']
            if v['@unit'] != '090' or v['@indicator'] != '0201010000000010000': raise ValueError('source unit/indicator mismatch')
            rows.append((v['@regionCode'], int(v['@time'][:4]), int(v['$'])))
        if set((r[0], r[1]) for r in rows) != {(c, y) for c in ('08201', '08212', '08220') for y in (2015, 2020)} or len(rows) != 6:
            raise ValueError('source population/time scope mismatch')
        conn.executemany('INSERT INTO population VALUES(?,?,?)', sorted(rows))
    return db


def run(sf, out, engine_commit, insight=None):
    if not re.fullmatch(r"[0-9a-f]{40}", engine_commit): raise ValueError("tested engine commit is required")
    out.mkdir(parents=True, exist_ok=True)
    db = prepare(out)
    manifest = {
        'manifestVersion': '1', 'engineCommit': engine_commit, 'question': 'What is the descriptive distribution of 2020 census population across these three selected municipalities?',
        'dataset': {'id': 'estat-selected-municipalities', 'version': 'census-2020-snapshot-20260927', 'sha256': exporter.file_hash(db)},
        'spec': {'reference': 'urn:stats-workbench:estat-selected-summary:v1', 'sha256': exporter.file_hash(RECIPE)},
        'period': {'start': '2020-10-01', 'end': '2020-10-01', 'basis': 'calendar-day'},
        'population': {'description': 'Finite selected group: Mito (08201), Hitachiota (08212), Tsukuba (08220); 2020 census resident populations', 'unit': 'municipality'},
        'geography': 'JP-08:selected-08201-08212-08220', 'source': 'https://dashboard.e-stat.go.jp/',
        'retrievedAt': '2026-09-27T16:50:36.651Z', 'license': 'PDL1.0',
        'variables': [{'name': 'population', 'unit': 'persons', 'definition': '2020-10-01 census resident population, including unspecified nationality/age', 'scope': 'estat-census-2020-selected-three'}],
        'method': 'DESCRIPTIVE_SUMMARY', 'missingnessPolicy': 'available-case',
        'assumptions': [{'name': 'COMPARABLE_POPULATION', 'status': 'PLAUSIBLE', 'basis': 'same census definition/date/unit; summary of the declared finite group'},
                        {'name': 'GEOGRAPHIC_BOUNDARY_CONTINUITY', 'status': 'UNKNOWN', 'basis': 'code continuity alone does not verify boundaries'}],
        'uncertainty': {'status': 'NOT_ESTIMATED', 'reason': 'No probabilistic sample or model; descriptive summary of three selected observations'},
    }
    (out / 'manifest.json').write_bytes(exporter.canonical(manifest) + b'\n')
    artifacts = []
    for n in (1, 2):
        result = out / f'result-{n}.json'
        env = {**os.environ, 'SF_POPULATION_DB': str(db), 'SF_POPULATION_RESULT': str(result)}
        subprocess.run([str(sf), 'run', '--recipe', str(RECIPE)], cwd=out, env=env, timeout=60, check=True, stdout=subprocess.DEVNULL)
        artifacts.append(exporter.export_summary(exporter.read_json(result), manifest, db, RECIPE, 'summary'))
    first, second = artifacts
    if first['parameters']['resultFingerprint'] != second['parameters']['resultFingerprint']: raise ValueError('rerun changed numerical results')
    expected = {'population.mean': 186981.0, 'population.min': 48602.0, 'population.max': 270685.0, 'population.q50': 241656.0}
    values = {r['metricId']: r.get('value') for r in first['results']}
    if any(values[k] != v for k, v in expected.items()): raise ValueError('public-data golden mismatch')
    (out / 'artifact.json').write_bytes(exporter.canonical(first) + b'\n')
    report = {'acceptanceVersion': '1', 'question': manifest['question'], 'sourceSnapshotSha256': SOURCE_HASH,
              'source': manifest['source'], 'publishedAt': None, 'retrievedAt': manifest['retrievedAt'],
              'analyticalArtifact': first, 'rerunResultFingerprint': second['parameters']['resultFingerprint'],
              'interpretation': 'DESCRIPTIVE_ONLY', 'samplingUncertainty': manifest['uncertainty'],
              'limitations': ['Three selected municipalities, not Japan', 'No cause or forecast established', 'Geographic boundary continuity unknown'],
              'engineAcceptance': 'NOT_RUN'}
    if insight:
        u = urlsplit(insight)
        if u.scheme != 'http' or u.hostname not in ('127.0.0.1', 'localhost', '::1') or u.username or u.password or u.path not in ('', '/') or u.query or u.fragment:
            raise ValueError('acceptance connects only to explicit local loopback HTTP origin')
        base = insight.rstrip('/')
        token = first['artifactHash']['value']
        subject = request(base, 'POST', '/subjects', {'contractVersion': '1', 'idempotencyKey': 'summary-subject-' + token, 'subject': {'namespace': 'stats-workbench.acceptance', 'id': token}, 'title': manifest['question']})
        sid = subject['subjectId']
        body = {'contractVersion': '1', 'idempotencyKey': 'summary-evidence-' + token, 'analyticalArtifacts': [first]}
        receipt = request(base, 'POST', '/subjects/' + sid + '/evidence', body)
        if receipt != request(base, 'POST', '/subjects/' + sid + '/evidence', body): raise ValueError('non-idempotent engine receipt')
        analysis = request(base, 'POST', '/subjects/' + sid + '/analyses', {'contractVersion': '1', 'idempotencyKey': 'summary-analysis-' + token, 'semanticAnalysisMode': 'DATASET_ANALYSIS', 'researchQuestion': manifest['question']})
        path = '/subjects/' + sid + '/analyses/' + analysis['analysisId']
        deadline = time.monotonic() + 30
        while analysis['status'] in ('queued', 'running') and time.monotonic() < deadline:
            time.sleep(.1); analysis = request(base, 'GET', path)
        if analysis['status'] != 'completed': raise ValueError('engine analysis did not complete')
        results = request(base, 'GET', path + '/results')
        if len(results['observations']) != len(first['results']): raise ValueError('missing neutral observations')
        if any(o['documentId'] != receipt['analyticalArtifacts'][0]['documentId'] for o in results['observations']): raise ValueError('observation lost evidence linkage')
        report.update(engineAcceptance='PASSED', evidenceReceipt=receipt, analysis=analysis, analysisResults=results)
    (out / 'report-ready.json').write_bytes(exporter.canonical(report) + b'\n')
    print(json.dumps({'resultFingerprint': first['parameters']['resultFingerprint'], 'engineAcceptance': report['engineAcceptance'], 'output': str(out)}))
    return first


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--engine-commit', required=True); p.add_argument('--sf', required=True, type=Path); p.add_argument('--out', required=True, type=Path); p.add_argument('--insight')
    a = p.parse_args()
    run(a.sf.resolve(), a.out.resolve(), a.engine_commit, a.insight)
