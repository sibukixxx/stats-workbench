"""Offline checks for the public snapshot and normalized SQLite bytes."""
import importlib.util
from pathlib import Path
import sqlite3
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('population', Path(__file__).with_name('run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

class PublicFixtureTest(unittest.TestCase):
    def test_source_and_normalized_input_are_reproducible(self):
        self.assertEqual(runner.exporter.file_hash(runner.SOURCE), runner.SOURCE_HASH)
        with tempfile.TemporaryDirectory() as tmp:
            first, second = Path(tmp) / 'first', Path(tmp) / 'second'
            first.mkdir(); second.mkdir()
            a, b = runner.prepare(first), runner.prepare(second)
            self.assertEqual(a.read_bytes(), b.read_bytes())
            with sqlite3.connect(a) as conn:
                rows = conn.execute('SELECT population FROM population WHERE year=2020 ORDER BY region_code').fetchall()
            self.assertEqual(rows, [(270685,), (48602,), (241656,)])
            with self.assertRaises(ValueError): runner.prepare(first)

if __name__ == '__main__': unittest.main()
