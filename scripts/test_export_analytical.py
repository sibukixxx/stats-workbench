import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

MODULE = Path(__file__).with_name('export_analytical.py')
spec = importlib.util.spec_from_file_location('export_analytical', MODULE)
exporter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(exporter)


class ExportTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.input = self.root / 'input.sqlite'
        self.recipe = self.root / 'recipe.toml'
        self.input.write_bytes(b'checked input')
        self.recipe.write_text('reviewed recipe')
        self.manifest = {
            'manifestVersion': '1', 'engineCommit': '64169a1219cca55c31c7a90514a50e50ff159ac1', 'question': 'Describe selected municipalities',
            'dataset': {'id': 'census', 'version': '2020', 'sha256': exporter.file_hash(self.input)},
            'spec': {'reference': 'urn:statforge:population-summary:v1', 'sha256': exporter.file_hash(self.recipe)},
            'period': {'start': '2020-10-01', 'end': '2020-10-01', 'basis': 'calendar-day'},
            'population': {'description': 'three selected municipalities', 'unit': 'municipality'},
            'geography': 'JP-08:selected-three', 'retrievedAt': '2026-09-27T16:50:36.651Z',
            'source': 'https://dashboard.e-stat.go.jp/', 'license': 'PDL1.0',
            'variables': [{'name': 'population', 'unit': 'persons', 'definition': 'resident population', 'scope': 'census-2020-three'}],
            'method': 'DESCRIPTIVE_SUMMARY', 'missingnessPolicy': 'available-case',
            'assumptions': [{'name': 'COMPARABLE_POPULATION', 'status': 'UNKNOWN', 'basis': 'boundaries not verified'}],
            'uncertainty': {'status': 'NOT_ESTIMATED', 'reason': 'finite descriptive group; no sampling design'},
        }
        self.run = {'execution': {'tool': 'StatForge', 'version': '0.1.0', 'started_at': '2026-10-11T03:30:00Z', 'recipe': 'pg://user:secret@host/private'},
                    'artifacts': {'summary': {'kind': 'Summary', 'values': {'columns': [{'name': 'population', 'count': 3, 'null_count': 0, 'mean': 186981.0, 'std': None, 'min': 48602.0, 'max': 270685.0, 'q25': 241656.0, 'q50': 241656.0, 'q75': 270685.0}]}}}}

    def export(self, manifest=None, run=None):
        return exporter.export_summary(run or self.run, manifest or self.manifest, self.input, self.recipe, 'summary')

    def test_golden_missing_privacy_and_repeated_export(self):
        a = self.export()
        self.assertEqual(a, self.export())
        mean = next(r for r in a['results'] if r['metricId'] == 'population.mean')
        self.assertEqual(mean['value'], 186981.0)
        missing = next(r for r in a['results'] if r['metricId'] == 'population.std')
        self.assertTrue(missing['missing'])
        self.assertNotIn('value', missing)
        self.assertEqual(a['parameters']['statisticalPlan']['uncertainty']['status'], 'NOT_ESTIMATED')
        self.assertNotIn('secret', json.dumps(a))
        self.assertNotIn(str(self.input), json.dumps(a))
        self.assertIn('ASSUMPTION_UNKNOWN', [f['code'] for f in a['qualityFlags']])
        self.assertEqual(a['artifactHash']['value'], exporter.digest({k: v for k, v in a.items() if k != 'artifactHash'}))

    def test_new_run_keeps_result_fingerprint_but_has_distinct_identity(self):
        b = copy.deepcopy(self.run)
        b['execution']['started_at'] = '2026-10-11T03:31:00Z'
        a, b = self.export(), self.export(run=b)
        self.assertEqual(a['parameters']['resultFingerprint'], b['parameters']['resultFingerprint'])
        self.assertNotEqual(a['id'], b['id'])

    def test_canonical_engine_preserves_values_and_v1_contract(self):
        current = copy.deepcopy(self.run)
        current['execution']['tool'] = 'Stats Workbench'
        artifact = self.export(run=current)
        self.assertEqual(artifact['artifactSchema'], 'insight-lab.analytical-artifact')
        self.assertEqual(artifact['schemaVersion'], '1')
        self.assertEqual(artifact['producer'], 'StatsWorkbench-summary-export')
        self.assertEqual(artifact['spec']['kind'], 'STATS_WORKBENCH_RECIPE')
        self.assertTrue(artifact['id'].startswith('stats-workbench-'))
        self.assertEqual(artifact['results'], self.export()['results'])
        self.assertEqual(artifact['provenance'], self.export()['provenance'])
        self.assertEqual(artifact, self.export(run=current))
        current['execution']['tool'] = 'unrelated'
        with self.assertRaises(ValueError): self.export(run=current)

    def test_reject_bad_plan_units_variables_population_assumptions_and_hashes(self):
        for mutate in [lambda m: m['variables'][0].pop('unit'), lambda m: m['variables'][0].update(name='absent'),
                       lambda m: m['variables'].append({'name': 'other', 'unit': 'persons', 'definition': 'other', 'scope': 'incomparable'}),
                       lambda m: m['assumptions'][0].update(status='VIOLATED'), lambda m: m['dataset'].update(sha256='0'*64),
                       lambda m: m['spec'].update(sha256='0'*64), lambda m: m.update(method='CAUSAL'),
                       lambda m: m.update(period={'start': '2021', 'end': '2020'}), lambda m: m.update(source='pg://user:secret@host')]:
            with self.subTest(mutate=mutate):
                m = copy.deepcopy(self.manifest)
                mutate(m)
                with self.assertRaises(ValueError): self.export(manifest=m)

    def test_reject_unsupported_malformed_nonfinite_and_bad_counters(self):
        for mutate in [lambda r: r['artifacts']['summary'].update(kind='Regression'),
                       lambda r: r['artifacts']['summary']['values']['columns'][0].update(count=2, null_count=3),
                       lambda r: r['artifacts']['summary']['values']['columns'][0].update(mean=float('nan')),
                       lambda r: r['artifacts']['summary']['values']['columns'][0].pop('mean')]:
            r = copy.deepcopy(self.run); mutate(r)
            with self.assertRaises(ValueError): self.export(run=r)

    def test_bounded_json_and_nonfinite_input(self):
        p = self.root / 'huge.json'; p.write_text(' ' * 20)
        with self.assertRaises(ValueError): exporter.read_json(p, limit=10)
        p.write_text('{"x":NaN}')
        with self.assertRaises(ValueError): exporter.read_json(p)


if __name__ == '__main__': unittest.main()
