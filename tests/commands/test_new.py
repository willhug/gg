import shutil, tempfile
import unittest
import os
from plumbum import local, FG

git_cmd = local['git']
ggDir = os.path.abspath(os.path.join(os.path.dirname(os.path.realpath(__file__)), os.pardir, os.pardir))
gg_main = os.path.abspath(os.path.join(ggDir, "gg", "main.py"))
py_cmd = local[ggDir + '/env/bin/python3.6']
gg_cmd = py_cmd[gg_main]


class TestNew(unittest.TestCase):
    def setUp(self):
        # Create a temporary directory
        self.test_dir = tempfile.mkdtemp()
        self.old_dir = os.path.dirname(os.path.realpath(__file__))

        os.chdir(self.test_dir)
        (git_cmd["init"]) & FG
        (local["touch"]["README.txt"]) & FG
        (git_cmd["add", "README.txt"]) & FG
        (git_cmd["commit", "-m" "Initial commit"]) & FG

    def tearDown(self):
        # Remove the directory after the test
        os.chdir(self.old_dir)
        shutil.rmtree(self.test_dir)

    def test_new(self):
        # Create a file in the temporary directory
        (gg_cmd["new", "-f", "test"]) & FG

        cmd = git_cmd["branch"]
        result = cmd()

        assert "test-part_1.0" in result
