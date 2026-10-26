"""Real binaries, loopback HTTP, fresh SQLite, no models or paid services."""
import importlib.util
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import unittest
import urllib.error
import urllib.request

spec = importlib.util.spec_from_file_location('population_runner', Path(__file__).with_name('run.py'))
runner = importlib.util.module_from_spec(spec); spec.loader.exec_module(runner)


class RealEngineAcceptance(unittest.TestCase):
    @unittest.skipUnless(os.environ.get('INSIGHT_BIN') and os.environ.get('STATS_WORKBENCH_BIN') and os.environ.get('STATS_WORKBENCH_COMMIT'), 'set INSIGHT_BIN, STATS_WORKBENCH_BIN and STATS_WORKBENCH_COMMIT')
    def test_real_producer_to_real_engine(self):
        with tempfile.TemporaryDirectory(prefix='stats-workbench-acceptance-') as tmp:
            root = Path(tmp)
            with socket.socket() as sock:
                sock.bind(('127.0.0.1', 0)); port = sock.getsockname()[1]
            base = f'http://127.0.0.1:{port}'
            with (root / 'engine.log').open('w') as log:
                engine = subprocess.Popen([os.environ['INSIGHT_BIN'], 'serve', '-host', '127.0.0.1', '-port', str(port), '-no-web', '-no-browser', '-db', str(root / 'engine.sqlite')], stdout=log, stderr=log)
                try:
                    ready = False
                    until = time.monotonic() + 30
                    while time.monotonic() < until and engine.poll() is None:
                        try:
                            with urllib.request.urlopen(base + '/api/public/v1/engine', timeout=1) as response:
                                info = json.load(response)
                            ready = True; break
                        except (urllib.error.URLError, TimeoutError): time.sleep(.1)
                    self.assertTrue(ready, 'local engine did not start')
                    self.assertFalse(info['modelBacked'], 'acceptance requires deterministic engine')
                    artifact = runner.run(Path(os.environ['STATS_WORKBENCH_BIN']), root / 'output', os.environ['STATS_WORKBENCH_COMMIT'], base)
                    report = json.loads((root / 'output/report-ready.json').read_text())
                    self.assertEqual(report['engineAcceptance'], 'PASSED')
                    self.assertEqual(len(report['analysisResults']['observations']), 7)
                    self.assertEqual(report['analysisResults']['findings'], [])
                    self.assertEqual(report['analysis']['status'], 'completed')
                    self.assertEqual(artifact['parameters']['resultFingerprint'], report['rerunResultFingerprint'])
                    self.assertTrue(report['analysis']['inputFingerprint'])
                    self.assertIsNone(report['publishedAt'])
                finally:
                    engine.terminate()
                    try: engine.wait(timeout=5)
                    except subprocess.TimeoutExpired: engine.kill(); engine.wait(timeout=5)


if __name__ == '__main__': unittest.main()
